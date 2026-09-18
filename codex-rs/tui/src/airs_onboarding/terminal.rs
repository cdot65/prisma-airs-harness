//! Bounded ownership of terminal modes before the main TUI starts.
use crossterm::cursor::Hide;
use crossterm::cursor::Show;
use crossterm::event::DisableBracketedPaste;
use crossterm::event::EnableBracketedPaste;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use std::io;
use std::io::IsTerminal;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::time::Duration;
use std::time::Instant;

static SCREEN_LOCK: Mutex<()> = Mutex::new(());

pub(super) struct TerminalGuard {
    _exclusive: MutexGuard<'static, ()>,
    alternate_screen: bool,
    raw_mode: bool,
    #[cfg(unix)]
    signals: Signals,
}

impl TerminalGuard {
    pub fn open() -> io::Result<Self> {
        if !io::stdin().is_terminal() || !io::stderr().is_terminal() {
            return Err(io::Error::other(
                "AIRS onboarding requires an interactive terminal",
            ));
        }
        let exclusive = SCREEN_LOCK
            .try_lock()
            .map_err(|_| io::Error::other("Another onboarding screen is active"))?;
        if crossterm::terminal::is_raw_mode_enabled()? {
            return Err(io::Error::other(
                "Finish the current terminal interaction before opening onboarding",
            ));
        }
        let mut guard = Self {
            _exclusive: exclusive,
            alternate_screen: false,
            raw_mode: false,
            #[cfg(unix)]
            signals: Signals::install()?,
        };
        crossterm::terminal::enable_raw_mode()?;
        guard.raw_mode = true;
        // Mark cleanup necessary before writing: a partial write may enter the screen.
        guard.alternate_screen = true;
        crossterm::execute!(
            io::stderr(),
            EnterAlternateScreen,
            Hide,
            EnableBracketedPaste
        )?;
        Ok(guard)
    }

    pub fn cancelled(&self) -> bool {
        #[cfg(unix)]
        return self.signals.cancelled();
        #[cfg(not(unix))]
        false
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let started = Instant::now();
        for _ in 0..1024 {
            if started.elapsed() >= Duration::from_millis(/*millis*/ 25)
                || !matches!(crossterm::event::poll(Duration::ZERO), Ok(true))
                || crossterm::event::read().is_err()
            {
                break;
            }
        }
        if self.alternate_screen {
            let _ = crossterm::execute!(
                io::stderr(),
                DisableBracketedPaste,
                Show,
                LeaveAlternateScreen
            );
        }
        if self.raw_mode {
            let _ = crossterm::terminal::disable_raw_mode();
        }
    }
}

#[cfg(unix)]
static CANCELLED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(/*v*/ false);

#[cfg(unix)]
extern "C" fn cancel(_: libc::c_int) {
    CANCELLED.store(/*val*/ true, std::sync::atomic::Ordering::Relaxed);
}

#[cfg(unix)]
struct Signals(Vec<(libc::c_int, libc::sigaction)>);

#[cfg(unix)]
impl Signals {
    fn install() -> io::Result<Self> {
        CANCELLED.store(/*val*/ false, std::sync::atomic::Ordering::Relaxed);
        let mut guard = Self(Vec::new());
        for signal in [
            libc::SIGINT,
            libc::SIGTERM,
            libc::SIGHUP,
            libc::SIGQUIT,
            libc::SIGTSTP,
            libc::SIGPIPE,
        ] {
            // sigaction is a C POD. The handler performs only a lock-free atomic store.
            let mut action: libc::sigaction = unsafe { std::mem::zeroed() };
            let mut previous: libc::sigaction = unsafe { std::mem::zeroed() };
            action.sa_sigaction = cancel as *const () as libc::sighandler_t;
            unsafe { libc::sigemptyset(&mut action.sa_mask) };
            if unsafe { libc::sigaction(signal, &action, &mut previous) } == -1 {
                return Err(io::Error::last_os_error());
            }
            guard.0.push((signal, previous));
        }
        Ok(guard)
    }

    fn cancelled(&self) -> bool {
        CANCELLED.load(std::sync::atomic::Ordering::Relaxed)
    }
}

#[cfg(unix)]
impl Drop for Signals {
    fn drop(&mut self) {
        for (signal, previous) in self.0.iter().rev() {
            let mut current: libc::sigaction = unsafe { std::mem::zeroed() };
            if unsafe { libc::sigaction(*signal, std::ptr::null(), &mut current) } == 0
                && current.sa_sigaction == cancel as *const () as libc::sighandler_t
            {
                unsafe { libc::sigaction(*signal, previous, std::ptr::null_mut()) };
            }
        }
    }
}
