//! Linux Desktop command births are held by pidfds in both the engine and an
//! independent helper. The strict trusted Bubblewrap/PID-namespace chain must
//! propagate root death; this module never substitutes a PID/PGID lookup.
use std::collections::BTreeMap;
use std::io;
use std::os::fd::AsRawFd;
use std::os::fd::OwnedFd;
use std::os::fd::RawFd;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::Weak;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;

use crate::linux_desktop_io as channel;
use channel::unavailable;

pub const DESKTOP_SUPERVISOR_ARG: &str = "--airs-desktop-command-supervisor";
const REGISTER: u64 = 1;
const CANCEL: u64 = 2;
const READY: channel::Record = [3, 0];
const MAX_ROOTS: usize = 1_024;
static DESKTOP_MODE: AtomicBool = AtomicBool::new(false);
static SPAWN: Mutex<State> = Mutex::new(State::Ordinary);
static ROOTS: Mutex<Vec<Weak<OwnedFd>>> = Mutex::new(Vec::new());

enum State {
    Ordinary,
    Active { socket: RawFd, next: u64 },
    Unavailable,
}

pub struct DesktopSupervisor {
    child: Child,
    socket: Option<OwnedFd>,
    pidfd: OwnedFd,
}

pub fn start_desktop_supervisor(executable: &Path) -> io::Result<DesktopSupervisor> {
    let mut command = Command::new(executable);
    command.arg(DESKTOP_SUPERVISOR_ARG);
    start(command)
}

fn start(mut command: Command) -> io::Result<DesktopSupervisor> {
    let mut state = SPAWN.lock().map_err(|_| unavailable())?;
    if !matches!(*state, State::Ordinary) {
        return Err(unavailable());
    }
    let (socket, child_socket) = channel::pair()?;
    command
        .env_clear()
        .stdin(Stdio::from(child_socket))
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    // SAFETY: only the fork-safe setsid syscall runs in the child.
    unsafe {
        command.pre_exec(|| {
            if libc::setsid() == -1 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let mut child = command.spawn()?;
    let setup = (|| {
        let pidfd = channel::pidfd(child.id() as libc::pid_t)?;
        if !matches!(channel::receive(socket.as_raw_fd())?, Some((READY, None))) {
            return Err(unavailable());
        }
        Ok(pidfd)
    })();
    let pidfd = match setup {
        Ok(pidfd) => pidfd,
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error);
        }
    };
    *state = State::Active {
        socket: socket.as_raw_fd(),
        next: 0,
    };
    DESKTOP_MODE.store(true, Ordering::Release);
    Ok(DesktopSupervisor {
        child,
        socket: Some(socket),
        pidfd,
    })
}

impl DesktopSupervisor {
    pub fn check_health(&self) -> io::Result<()> {
        if channel::poll(self.pidfd.as_raw_fd(), 0)? {
            if let Ok(mut state) = SPAWN.lock() {
                *state = State::Unavailable;
            }
            kill_local_roots()?;
            return Err(unavailable());
        }
        require_desktop_command_supervision()
    }
}

impl Drop for DesktopSupervisor {
    fn drop(&mut self) {
        if let Ok(mut state) = SPAWN.lock() {
            *state = State::Unavailable;
            self.socket.take();
        }
        let _ = kill_local_roots();
        if channel::poll(self.pidfd.as_raw_fd(), channel::TIMEOUT_MS).unwrap_or(false) {
            let _ = self.child.wait();
        }
    }
}

/// Bootstrap must refuse Desktop mode without the independent lifetime guard.
pub fn require_desktop_command_supervision() -> io::Result<()> {
    let state = SPAWN.lock().map_err(|_| unavailable())?;
    match *state {
        State::Active { socket, .. } if !channel::poll(socket, 0)? => Ok(()),
        _ => Err(unavailable()),
    }
}

pub(crate) fn desktop_mode_enabled() -> bool {
    DESKTOP_MODE.load(Ordering::Acquire)
}

pub(crate) struct DesktopSpawnGuard {
    state: MutexGuard<'static, State>,
    parent: OwnedFd,
    child: OwnedFd,
    registration: Registration,
    received: AtomicBool,
}

#[derive(Clone, Copy)]
pub(crate) struct Registration {
    socket: RawFd,
    parent_return: RawFd,
    child_return: RawFd,
    owner: libc::pid_t,
    sequence: u64,
}

pub(crate) fn prepare_spawn() -> io::Result<Option<DesktopSpawnGuard>> {
    let mut state = SPAWN.lock().map_err(|_| unavailable())?;
    let State::Active { socket, next } = &mut *state else {
        return if matches!(*state, State::Ordinary) {
            Ok(None)
        } else {
            Err(unavailable())
        };
    };
    *next = next.checked_add(1).ok_or_else(unavailable)?;
    let (parent, child) = channel::pair()?;
    let registration = Registration {
        socket: *socket,
        parent_return: parent.as_raw_fd(),
        child_return: child.as_raw_fd(),
        owner: unsafe { libc::getpid() },
        sequence: *next,
    };
    Ok(Some(DesktopSpawnGuard {
        state,
        parent,
        child,
        registration,
        received: AtomicBool::new(false),
    }))
}

impl DesktopSpawnGuard {
    pub(crate) fn registration(&self) -> Registration {
        self.registration
    }

    pub(crate) fn command(&self) -> io::Result<DesktopCommand> {
        let expected = [REGISTER, self.registration.sequence];
        let Some((record, Some(pidfd))) = channel::receive(self.parent.as_raw_fd())? else {
            return Err(unavailable());
        };
        if record != expected {
            return Err(unavailable());
        }
        let pidfd = Arc::new(pidfd);
        let mut roots = ROOTS.lock().map_err(|_| unavailable())?;
        roots.retain(|root| root.strong_count() != 0);
        if roots.len() >= MAX_ROOTS {
            channel::signal(&pidfd, libc::SIGKILL)?;
            return Err(unavailable());
        }
        roots.push(Arc::downgrade(&pidfd));
        self.received.store(true, Ordering::Release);
        Ok(DesktopCommand(pidfd))
    }
}

impl Drop for DesktopSpawnGuard {
    fn drop(&mut self) {
        // Keep the registration lock and both return descriptors until spawn
        // and the parent's exact pidfd receipt have completed.
        let _ = &self.child;
        if !self.received.load(Ordering::Acquire) {
            // Interrupted spawn/receipt must stop the independent cleanup loop,
            // including any root already admitted before the parent saw exec.
            unsafe { libc::shutdown(self.registration.socket, libc::SHUT_RDWR) };
            *self.state = State::Unavailable;
            let _ = kill_local_roots();
        }
    }
}

pub(crate) struct DesktopCommand(Arc<OwnedFd>);
impl DesktopCommand {
    pub(crate) fn signal(&self, signal: i32) -> io::Result<()> {
        channel::signal(&self.0, signal)
    }
}

impl Registration {
    /// Stack-only protocol and syscalls: called after fork, before user exec.
    pub(crate) fn register(self) -> io::Result<()> {
        // A trusted nonprivileged helper is required until Bubblewrap owns its
        // PID namespace. SIGKILL covers interrupted setup, not the whole proof.
        if unsafe { libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) } != 0
            || unsafe { libc::getppid() } != self.owner
        {
            return Err(unavailable());
        }
        let pidfd = channel::pidfd(unsafe { libc::getpid() })?;
        let record = [REGISTER, self.sequence];
        channel::send(self.socket, record, Some(pidfd.as_raw_fd()))?;
        if !matches!(channel::receive(self.socket)?, Some((ack, None)) if ack == record)
            || unsafe { libc::getppid() } != self.owner
        {
            return Err(unavailable());
        }
        channel::send(self.child_return, record, Some(pidfd.as_raw_fd()))?;
        // No user command inherits a supervisor or registration capability.
        unsafe {
            libc::close(self.socket);
            libc::close(self.parent_return);
            libc::close(self.child_return);
        }
        Ok(())
    }
}

pub fn cancel_desktop_commands() -> io::Result<()> {
    let mut state = SPAWN.lock().map_err(|_| unavailable())?;
    let result = match &mut *state {
        State::Active { socket, next } => (|| {
            *next = next.checked_add(1).ok_or_else(unavailable)?;
            let record = [CANCEL, *next];
            channel::send(*socket, record, None)?;
            if !matches!(channel::receive(*socket)?, Some((ack, None)) if ack == record) {
                return Err(unavailable());
            }
            Ok(())
        })(),
        _ => Err(unavailable()),
    };
    let local = kill_local_roots();
    if result.is_err() || local.is_err() {
        *state = State::Unavailable;
    }
    result.and(local)
}

fn kill_local_roots() -> io::Result<()> {
    let mut roots = ROOTS.lock().map_err(|_| unavailable())?;
    let owned: Vec<_> = roots.iter().filter_map(Weak::upgrade).collect();
    let result = kill_roots(owned.iter().map(AsRef::as_ref));
    roots.retain(|root| root.strong_count() != 0);
    result
}

fn kill_roots<'a>(roots: impl Iterator<Item = &'a OwnedFd> + Clone) -> io::Result<()> {
    let mut first_error = None;
    for root in roots.clone() {
        if let Err(error) = channel::signal(root, libc::SIGKILL)
            && first_error.is_none()
        {
            first_error = Some(error);
        }
    }
    let deadline = Instant::now() + Duration::from_millis(channel::TIMEOUT_MS as u64);
    for root in roots {
        let remaining = deadline
            .saturating_duration_since(Instant::now())
            .as_millis() as i32;
        let result = match channel::poll(root.as_raw_fd(), remaining) {
            Ok(true) => Ok(()),
            Ok(false) => Err(unavailable()),
            Err(error) => Err(error),
        };
        if let Err(error) = result
            && first_error.is_none()
        {
            first_error = Some(error);
        }
    }
    first_error.map_or(Ok(()), Err)
}

/// Private early dispatch; only the same executable and its private socket can
/// launch this helper. It receives process capabilities, never command data.
pub fn run_desktop_supervisor() -> io::Result<()> {
    let owner = unsafe { libc::getppid() };
    if owner <= 1 || std::fs::read_link(format!("/proc/{owner}/exe"))? != std::env::current_exe()? {
        return Err(unavailable());
    }
    let owner_pidfd = channel::pidfd(owner)?;
    let mut roots = BTreeMap::new();
    let mut previous = 0;
    channel::send(0, READY, None)?;
    let result = (|| {
        while !channel::poll(owner_pidfd.as_raw_fd(), 0)? {
            roots.retain(|_, fd: &mut OwnedFd| {
                !matches!(channel::poll(fd.as_raw_fd(), 0), Ok(true))
            });
            if !channel::poll(0, 25)? {
                continue;
            }
            let Some((record, descriptor)) = channel::receive(0)? else {
                break;
            };
            if record[1] <= previous {
                return Err(unavailable());
            }
            match (record[0], descriptor) {
                (REGISTER, Some(pidfd)) if roots.len() < MAX_ROOTS => {
                    validate_root(&pidfd, owner)?;
                    roots.insert(record[1], pidfd);
                }
                (CANCEL, None) => {
                    kill_roots(roots.values())?;
                    roots.clear();
                }
                _ => return Err(unavailable()),
            }
            previous = record[1];
            channel::send(0, record, None)?;
        }
        Ok(())
    })();
    kill_roots(roots.values())?;
    result
}

fn validate_root(pidfd: &OwnedFd, owner: libc::pid_t) -> io::Result<()> {
    channel::signal(pidfd, 0)?;
    let info = std::fs::read_to_string(format!("/proc/self/fdinfo/{}", pidfd.as_raw_fd()))?;
    let pid: libc::pid_t = info
        .lines()
        .find_map(|line| line.strip_prefix("Pid:\t"))
        .and_then(|pid| pid.parse().ok())
        .filter(|pid| *pid > 1)
        .ok_or_else(unavailable)?;
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat"))?;
    let parent = stat
        .rsplit_once(')')
        .and_then(|(_, fields)| fields.split_whitespace().nth(1))
        .and_then(|parent| parent.parse::<libc::pid_t>().ok());
    // Both checks are against an already retained kernel identity. Exit/reuse
    // during inspection cannot redirect signaling to the process at a new PID.
    if parent != Some(owner) || channel::poll(pidfd.as_raw_fd(), 0)? {
        return Err(unavailable());
    }
    Ok(())
}

#[cfg(test)]
#[path = "linux_desktop_supervisor_tests.rs"]
mod tests;
