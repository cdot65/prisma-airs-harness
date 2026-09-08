//! Restore terminal state and discard unread secret input on every normal exit.
use std::io;
use std::io::Write;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::time::Duration;
use std::time::Instant;

static PROMPT_LOCK: Mutex<()> = Mutex::new(());

pub(super) struct SecretTerminal {
    state: platform::TerminalState,
    signals: platform::Cancellation,
    bracketed_paste: bool,
    _exclusive: MutexGuard<'static, ()>,
}

impl SecretTerminal {
    pub(super) fn open() -> io::Result<Self> {
        let exclusive = PROMPT_LOCK
            .try_lock()
            .map_err(|_| io::Error::other("Another secure input prompt is already active"))?;
        let state = platform::TerminalState::capture()?;
        let signals = platform::Cancellation::install()?;
        let mut terminal = Self {
            state,
            signals,
            bracketed_paste: false,
            _exclusive: exclusive,
        };
        crossterm::terminal::enable_raw_mode()?;
        // The Windows console event source does not implement bracketed paste.
        // Do not emit unsupported VT controls into legacy consoles.
        #[cfg(unix)]
        {
            terminal.bracketed_paste = true;
            crossterm::execute!(io::stderr(), crossterm::event::EnableBracketedPaste)?;
        }
        Ok(terminal)
    }

    pub(super) fn cancelled(&self) -> bool {
        self.signals.cancelled()
    }
}

impl Drop for SecretTerminal {
    fn drop(&mut self) {
        self.state.discard_input();
        // Event readers can have buffered keys beyond the native input queue.
        // Drain those too before a subsequent TUI takes over, with bounded work.
        let started = Instant::now();
        for _ in 0..32_768 {
            if started.elapsed() >= Duration::from_millis(50)
                || !matches!(crossterm::event::poll(Duration::ZERO), Ok(true))
                || crossterm::event::read().is_err()
            {
                break;
            }
        }
        if self.bracketed_paste {
            let _ = crossterm::execute!(io::stderr(), crossterm::event::DisableBracketedPaste);
        }
        self.state.restore();
        let _ = writeln!(io::stderr());
    }
}

#[cfg(unix)]
mod platform {
    use std::io;
    use std::sync::atomic::AtomicI32;
    use std::sync::atomic::Ordering;

    static CANCEL_SIGNAL: AtomicI32 = AtomicI32::new(0);

    extern "C" fn cancel(signal: libc::c_int) {
        // Lock-free atomic storage is the only operation in the signal handler.
        CANCEL_SIGNAL.store(signal, Ordering::Relaxed);
    }

    pub(super) struct Cancellation {
        previous: Vec<(libc::c_int, libc::sigaction)>,
    }

    impl Cancellation {
        pub(super) fn install() -> io::Result<Self> {
            CANCEL_SIGNAL.store(0, Ordering::Relaxed);
            let mut guard = Self {
                previous: Vec::new(),
            };
            for signal in [
                libc::SIGINT,
                libc::SIGTERM,
                libc::SIGHUP,
                libc::SIGQUIT,
                libc::SIGTSTP,
                libc::SIGPIPE,
            ] {
                // sigaction is a C POD structure; every field is initialized.
                let mut action: libc::sigaction = unsafe { std::mem::zeroed() };
                let mut previous: libc::sigaction = unsafe { std::mem::zeroed() };
                action.sa_sigaction = cancel as *const () as libc::sighandler_t;
                unsafe { libc::sigemptyset(&mut action.sa_mask) };
                if unsafe { libc::sigaction(signal, &action, &mut previous) } == -1 {
                    return Err(io::Error::last_os_error());
                }
                guard.previous.push((signal, previous));
            }
            Ok(guard)
        }

        pub(super) fn cancelled(&self) -> bool {
            CANCEL_SIGNAL.load(Ordering::Relaxed) != 0
        }
    }

    impl Drop for Cancellation {
        fn drop(&mut self) {
            for (signal, previous) in self.previous.iter().rev() {
                let mut current: libc::sigaction = unsafe { std::mem::zeroed() };
                if unsafe { libc::sigaction(*signal, std::ptr::null(), &mut current) } == 0
                    && current.sa_sigaction == cancel as *const () as libc::sighandler_t
                {
                    // A concurrent component's newer handler must not be
                    // overwritten. CLI dispatch keeps OIDC and this prompt serial.
                    unsafe { libc::sigaction(*signal, previous, std::ptr::null_mut()) };
                }
            }
        }
    }

    pub(super) struct TerminalState(libc::termios);

    impl TerminalState {
        pub(super) fn capture() -> io::Result<Self> {
            let mut original = std::mem::MaybeUninit::uninit();
            if unsafe { libc::tcgetattr(libc::STDIN_FILENO, original.as_mut_ptr()) } == -1 {
                return Err(io::Error::last_os_error());
            }
            Ok(Self(unsafe { original.assume_init() }))
        }

        pub(super) fn discard_input(&self) {
            unsafe { libc::tcflush(libc::STDIN_FILENO, libc::TCIFLUSH) };
        }

        pub(super) fn restore(&self) {
            self.discard_input();
            // Also clear crossterm's saved raw-mode state before restoring the
            // precise termios snapshot (including an unusual original mode).
            let _ = crossterm::terminal::disable_raw_mode();
            unsafe { libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, &self.0) };
        }
    }
}

#[cfg(windows)]
mod platform {
    use std::io;
    use windows_sys::Win32::Foundation::HANDLE;
    use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
    use windows_sys::Win32::System::Console::FlushConsoleInputBuffer;
    use windows_sys::Win32::System::Console::GetConsoleMode;
    use windows_sys::Win32::System::Console::GetStdHandle;
    use windows_sys::Win32::System::Console::STD_INPUT_HANDLE;
    use windows_sys::Win32::System::Console::SetConsoleMode;

    pub(super) struct Cancellation;

    impl Cancellation {
        pub(super) fn install() -> io::Result<Self> {
            Ok(Self)
        }

        pub(super) fn cancelled(&self) -> bool {
            false
        }
    }

    pub(super) struct TerminalState {
        input: HANDLE,
        mode: u32,
    }

    impl TerminalState {
        pub(super) fn capture() -> io::Result<Self> {
            let input = unsafe { GetStdHandle(STD_INPUT_HANDLE) };
            let mut mode = 0;
            if input == 0
                || input == INVALID_HANDLE_VALUE
                || unsafe { GetConsoleMode(input, &mut mode) } == 0
            {
                return Err(io::Error::last_os_error());
            }
            Ok(Self { input, mode })
        }

        pub(super) fn discard_input(&self) {
            unsafe { FlushConsoleInputBuffer(self.input) };
        }

        pub(super) fn restore(&self) {
            self.discard_input();
            // crossterm::disable_raw_mode unconditionally adds console flags,
            // so restore the captured bitmask directly on this platform.
            unsafe { SetConsoleMode(self.input, self.mode) };
        }
    }
}
