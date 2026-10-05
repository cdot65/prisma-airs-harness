//! Desktop-only command lifetime supervision, independent of the engine group.
//!
//! The private descriptors carry process identities only. Children register
//! before exec; they never inherit these descriptors into the user command.
use std::io;
use std::os::fd::AsRawFd;
use std::os::fd::RawFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Child;
use std::process::ChildStdin;
use std::process::ChildStdout;
use std::process::Command;
use std::process::Stdio;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;

use crate::desktop_monitor::Monitor;
use crate::desktop_process_tree::AuditToken;
use crate::desktop_process_tree::ProcessTree;
use crate::desktop_process_tree::audit_token;
use crate::desktop_process_tree::process_info;
use crate::desktop_supervisor_io::read_exact_bounded;
use crate::desktop_supervisor_io::read_exact_with_timeout;
use crate::desktop_supervisor_io::set_cloexec;
use crate::desktop_supervisor_io::set_nonblocking;
use crate::desktop_supervisor_io::write_bounded;

/// Private early-dispatch argument; it must be handled before engine bootstrap.
pub const DESKTOP_SUPERVISOR_ARG: &str = "--airs-desktop-command-supervisor";
const READY: u8 = 0xff;
const CANCEL_TIMEOUT_MS: libc::c_int = 6_000;
// A real audit token always has a positive PID. The reserved PID0 control has a
// fixed tag and fresh bounded sequence, with no user data or process target.
const CANCEL_TAG: &[u8; 16] = b"AIRS_CANCEL_V1\0\0";
const MAX_CONTROLS: u64 = 1_024;
const MAX_ROOTS: usize = 1_024;
static DESKTOP_MODE: AtomicBool = AtomicBool::new(false);
static UNAVAILABLE: AtomicBool = AtomicBool::new(false);
static CANCEL_SEQUENCE: AtomicU64 = AtomicU64::new(0);
static SPAWN: Mutex<State> = Mutex::new(State::Ordinary);

#[derive(Clone, Copy)]
pub(crate) struct Registration {
    input: RawFd,
    output: RawFd,
    backup_input: RawFd,
    backup_output: RawFd,
    parent: libc::pid_t,
}
enum State {
    Ordinary,
    Active(Registration),
    Unavailable,
}

/// Keeps the secret-free independent supervisor alive for a desktop engine.
/// Dropping this guard closes the private lifetime pipe and waits for cleanup.
pub struct DesktopSupervisor {
    child: Child,
    input: Option<ChildStdin>,
    output: Option<ChildStdout>,
    monitor: Option<Monitor>,
}

/// Start desktop command supervision before any command is admitted.
/// The executable must early-dispatch [`DESKTOP_SUPERVISOR_ARG`] to the runner.
pub fn start_desktop_supervisor(executable: &Path) -> io::Result<DesktopSupervisor> {
    let mut command = Command::new(executable);
    command.arg(DESKTOP_SUPERVISOR_ARG);
    start(command, ReadyProtocol::Exact)
}

/// Cancel currently supervised desktop commands, including discovered children
/// in separate sessions, while keeping supervision available for later turns.
/// Call this before interrupting the engine's original command groups so their
/// live ancestry remains available to both independent cleanup loops.
pub fn cancel_desktop_commands() -> io::Result<()> {
    let mut state = SPAWN.lock().map_err(|_| unavailable())?;
    let registration = match *state {
        State::Active(registration) if !UNAVAILABLE.load(Ordering::Acquire) => registration,
        State::Ordinary | State::Active(_) | State::Unavailable => return Err(unavailable()),
    };
    let result = (|| {
        if unsafe { libc::getpid() } != registration.parent {
            return Err(unavailable());
        }
        let sequence = CANCEL_SEQUENCE
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |previous| {
                (previous < MAX_CONTROLS).then_some(previous + 1)
            })
            .map_err(|_| unavailable())?
            + 1;
        let mut record = [0; std::mem::size_of::<AuditToken>()];
        record[..16].copy_from_slice(CANCEL_TAG);
        record[24..].copy_from_slice(&sequence.to_le_bytes());
        for (input, output) in [
            (registration.backup_input, registration.backup_output),
            (registration.input, registration.output),
        ] {
            write_bounded(input, &record)?;
            let mut ack = [0xff; std::mem::size_of::<AuditToken>()];
            read_exact_with_timeout(output, &mut ack, CANCEL_TIMEOUT_MS)?;
            if ack != record {
                return Err(unavailable());
            }
        }
        if UNAVAILABLE.load(Ordering::Acquire) {
            return Err(unavailable());
        }
        Ok(())
    })();
    if result.is_err() {
        // An unconsumed control ACK must never be accepted by a later call.
        *state = State::Unavailable;
        mark_unavailable();
    }
    result
}

enum ReadyProtocol {
    Exact,
    #[cfg(test)]
    TestHarness,
}
fn start(mut command: Command, protocol: ReadyProtocol) -> io::Result<DesktopSupervisor> {
    let mut state = SPAWN.lock().map_err(|_| unavailable())?;
    if !matches!(*state, State::Ordinary) {
        return Err(unavailable());
    }
    command
        .env_clear()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    // SAFETY: setsid is fork-safe and keeps the broker outside the engine PGID.
    unsafe {
        command.pre_exec(|| {
            if libc::setsid() == -1 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let mut child = command.spawn()?;
    let input = child.stdin.take().ok_or_else(unavailable)?;
    let output = child.stdout.take().ok_or_else(unavailable)?;
    let ready = (|| {
        set_cloexec(input.as_raw_fd())?;
        set_cloexec(output.as_raw_fd())?;
        set_nonblocking(input.as_raw_fd())?;
        // Only utility subprocess fixtures have a libtest header before ready.
        let limit = match protocol {
            ReadyProtocol::Exact => 1,
            #[cfg(test)]
            ReadyProtocol::TestHarness => 256,
        };
        for _ in 0..limit {
            let mut byte = [0];
            read_exact_bounded(output.as_raw_fd(), &mut byte)?;
            if byte[0] == READY {
                return Ok(());
            }
        }
        Err(unavailable())
    })();
    if let Err(error) = ready {
        drop(input);
        drop(output);
        let _ = child.kill();
        let _ = child.wait();
        return Err(error);
    }
    let monitor = match Monitor::start(&child) {
        Ok(monitor) => monitor,
        Err(error) => {
            drop(input);
            drop(output);
            let _ = child.kill();
            let _ = child.wait();
            return Err(error);
        }
    };
    let (backup_input, backup_output) = monitor.descriptors()?;
    *state = State::Active(Registration {
        input: input.as_raw_fd(),
        output: output.as_raw_fd(),
        backup_input,
        backup_output,
        parent: unsafe { libc::getpid() },
    });
    DESKTOP_MODE.store(true, Ordering::Release);
    Ok(DesktopSupervisor {
        child,
        input: Some(input),
        output: Some(output),
        monitor: Some(monitor),
    })
}

impl DesktopSupervisor {
    /// Fail closed if either supervisor has lost its ability to clean commands.
    pub fn check_health(&self) -> io::Result<()> {
        if UNAVAILABLE.load(Ordering::Acquire) {
            Err(unavailable())
        } else {
            Ok(())
        }
    }
}

impl Drop for DesktopSupervisor {
    fn drop(&mut self) {
        if let Ok(mut state) = SPAWN.lock() {
            *state = State::Unavailable;
            mark_unavailable();
            self.input.take();
            self.output.take();
        }
        self.monitor.take();
        // The broker has its own bounded cleanup. Never kill it mid-cleanup.
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            match self.child.try_wait() {
                Ok(Some(_)) | Err(_) => return,
                Ok(None) => std::thread::sleep(Duration::from_millis(10)),
            }
        }
    }
}

pub(crate) fn mark_unavailable() {
    UNAVAILABLE.store(true, Ordering::Release);
}

pub(crate) fn desktop_mode_enabled() -> bool {
    DESKTOP_MODE.load(Ordering::Acquire)
}

pub(crate) struct DesktopSpawnGuard(MutexGuard<'static, State>);
impl DesktopSpawnGuard {
    pub(crate) fn registration(&self) -> Registration {
        match *self.0 {
            State::Active(registration) => registration,
            State::Ordinary | State::Unavailable => unreachable!(),
        }
    }
}

/// Held only across synchronous spawn; never acquired inside the forked child.
pub(crate) fn prepare_spawn() -> io::Result<Option<DesktopSpawnGuard>> {
    let state = SPAWN.lock().map_err(|_| unavailable())?;
    match *state {
        State::Ordinary => Ok(None),
        State::Active(_) if !UNAVAILABLE.load(Ordering::Acquire) => {
            Ok(Some(DesktopSpawnGuard(state)))
        }
        State::Active(_) => Err(unavailable()),
        State::Unavailable => Err(unavailable()),
    }
}

impl Registration {
    /// Only fork-safe syscalls and stack buffers are used before user exec.
    pub(crate) fn register(self) -> io::Result<()> {
        let pid = unsafe { libc::getpid() };
        if unsafe { libc::getppid() } != self.parent || unsafe { libc::getpgrp() } != pid {
            return Err(unavailable());
        }
        let token = audit_token(pid)?;
        let bytes = unsafe {
            std::slice::from_raw_parts(
                (&raw const token).cast::<u8>(),
                std::mem::size_of::<AuditToken>(),
            )
        };
        for (input, output) in [
            (self.backup_input, self.backup_output),
            (self.input, self.output),
        ] {
            write_bounded(input, bytes)?;
            let mut ack = [0_u8; std::mem::size_of::<AuditToken>()];
            read_exact_bounded(output, &mut ack)?;
            let accepted = unsafe { std::ptr::read_unaligned(ack.as_ptr().cast::<AuditToken>()) };
            if accepted != token || unsafe { libc::getppid() } != self.parent {
                return Err(unavailable());
            }
        }
        unsafe {
            libc::close(self.input);
            libc::close(self.output);
            libc::close(self.backup_input);
            libc::close(self.backup_output);
        }
        Ok(())
    }
}

/// Run the private helper. It reads fixed-size identities from stdin and stops
/// owned processes when the engine's lifetime pipe closes. No credentials,
/// command text, environment, project paths, or conversation data are received.
pub fn run_desktop_supervisor() -> io::Result<()> {
    let owner = unsafe { libc::getppid() };
    if owner <= 1 || !private_owner(owner)? {
        return Err(unavailable());
    }
    set_nonblocking(libc::STDOUT_FILENO)?;
    if unsafe { libc::write(1, [READY].as_ptr().cast(), 1) } != 1 {
        return Err(unavailable());
    }
    run_registration_loop(0, 1, owner, || unsafe { libc::getppid() } == owner)
}

pub(crate) fn run_registration_loop(
    input: RawFd,
    output: RawFd,
    owner: libc::pid_t,
    remains_alive: impl Fn() -> bool,
) -> io::Result<()> {
    let mut tree = ProcessTree::new();
    let result = (|| {
        let mut pending = [0_u8; std::mem::size_of::<AuditToken>()];
        let mut used = 0;
        let mut previous_control = 0;
        loop {
            if !remains_alive() {
                return Ok(());
            }
            let mut poll = libc::pollfd {
                fd: input,
                events: libc::POLLIN,
                revents: 0,
            };
            let ready = unsafe { libc::poll(&raw mut poll, 1, 25) };
            if ready < 0 {
                if io::Error::last_os_error().kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(unavailable());
            }
            if ready > 0 {
                let read = unsafe {
                    libc::read(
                        input,
                        pending[used..].as_mut_ptr().cast(),
                        pending.len() - used,
                    )
                };
                if read == 0 {
                    return Ok(());
                }
                if read < 0 {
                    return Err(unavailable());
                }
                used += read as usize;
                if used == pending.len() {
                    let token =
                        unsafe { std::ptr::read_unaligned(pending.as_ptr().cast::<AuditToken>()) };
                    if token.pid() == 0 {
                        let sequence = u64::from_le_bytes(
                            pending[24..].try_into().map_err(|_| unavailable())?,
                        );
                        if pending[..16] != *CANCEL_TAG
                            || pending[16..24] != [0; 8]
                            || sequence != previous_control + 1
                            || sequence > MAX_CONTROLS
                        {
                            return Err(unavailable());
                        }
                        tree.cleanup()?;
                        previous_control = sequence;
                    } else {
                        let info = process_info(token.pid()).ok_or_else(unavailable)?;
                        if tree.root_count() >= MAX_ROOTS
                            || token.pid() <= 1
                            || info.pbi_ppid != owner as u32
                            || info.pbi_pgid != token.pid() as u32
                            || info.pbi_uid != unsafe { libc::getuid() }
                        {
                            return Err(unavailable());
                        }
                        tree.register(token, &info)?;
                    }
                    // Correlate the ACK to this exact pre-exec birth. A child
                    // killed before reading cannot authorize the next spawn.
                    write_bounded(output, &pending)?;
                    used = 0;
                }
            }
            tree.discover()?;
        }
    })();
    tree.cleanup()?;
    result
}

fn private_owner(owner: libc::pid_t) -> io::Result<bool> {
    // The hidden dispatch cannot be used as an arbitrary standalone signalling
    // API. Only this same executable, with two private anonymous pipes, owns it.
    for fd in [libc::STDIN_FILENO, libc::STDOUT_FILENO] {
        let mut info: libc::stat = unsafe { std::mem::zeroed() };
        if unsafe { libc::fstat(fd, &raw mut info) } != 0
            || info.st_mode & libc::S_IFMT != libc::S_IFIFO
        {
            return Ok(false);
        }
    }
    let executable = std::env::current_exe()?.canonicalize()?;
    let mut path = [0_u8; 4_096];
    let size = unsafe { libc::proc_pidpath(owner, path.as_mut_ptr().cast(), path.len() as u32) };
    if size <= 0 {
        return Ok(false);
    }
    let end = path
        .iter()
        .position(|byte| *byte == 0)
        .ok_or_else(unavailable)?;
    let parent = Path::new(std::ffi::OsStr::from_bytes(&path[..end])).canonicalize()?;
    Ok(parent == executable)
}

fn unavailable() -> io::Error {
    io::Error::other("desktop command supervision unavailable")
}

#[cfg(test)]
#[path = "desktop_supervisor_tests.rs"]
mod tests;
