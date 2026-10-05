//! Bounded Darwin descendant cleanup with kernel-checked process identities.
//!
//! Darwin no longer supports kqueue NOTE_TRACK. Polling cannot retain a process
//! that double-forks, detaches and reparents before any ancestry snapshot. This
//! is command lifetime cleanup, not a general hostile-process containment API.
use std::collections::BTreeMap;
use std::io;
use std::time::Duration;
use std::time::Instant;

const MAX_PROCESSES: usize = 4_096;
const TASK_AUDIT_TOKEN: libc::c_uint = 15;
// sys/proc_info.h; libc exposes proc_listpids but omits this selector.
const PROC_PPID_ONLY: libc::c_uint = 6;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct AuditToken {
    values: [u32; 8],
}

#[cfg(test)]
#[path = "desktop_process_tree_tests.rs"]
mod tests;
impl AuditToken {
    pub(crate) fn pid(self) -> libc::pid_t {
        self.values[5] as libc::pid_t
    }
    fn signal(self, signal: libc::c_int) -> io::Result<()> {
        let result = unsafe { proc_signal_with_audittoken(&self, signal) };
        if result == 0 {
            Ok(())
        } else {
            Err(io::Error::from_raw_os_error(result))
        }
    }
}

unsafe extern "C" {
    static mach_task_self_: libc::mach_port_t;
    fn task_name_for_pid(
        task: libc::mach_port_t,
        pid: libc::pid_t,
        name: *mut libc::mach_port_t,
    ) -> libc::c_int;
    fn mach_port_deallocate(task: libc::mach_port_t, name: libc::mach_port_t) -> libc::c_int;
    fn proc_signal_with_audittoken(token: *const AuditToken, signal: libc::c_int) -> libc::c_int;
}

pub(crate) fn audit_token(pid: libc::pid_t) -> io::Result<AuditToken> {
    let own = unsafe { mach_task_self_ };
    let mut task = own;
    if pid != unsafe { libc::getpid() }
        && unsafe { task_name_for_pid(own, pid, &raw mut task) } != 0
    {
        return Err(io::Error::other("desktop process identity unavailable"));
    }
    let mut token = AuditToken { values: [0; 8] };
    let mut count = 8;
    let result = unsafe {
        libc::task_info(
            task,
            TASK_AUDIT_TOKEN,
            token.values.as_mut_ptr().cast(),
            &raw mut count,
        )
    };
    if task != own {
        unsafe {
            mach_port_deallocate(own, task);
        }
    }
    if result != 0 || count != 8 || token.pid() != pid {
        return Err(io::Error::other("desktop process identity unavailable"));
    }
    Ok(token)
}

pub(crate) fn process_info(pid: libc::pid_t) -> Option<libc::proc_bsdinfo> {
    let mut info: libc::proc_bsdinfo = unsafe { std::mem::zeroed() };
    let size = std::mem::size_of_val(&info) as libc::c_int;
    (unsafe { libc::proc_pidinfo(pid, libc::PROC_PIDTBSDINFO, 0, (&raw mut info).cast(), size) }
        == size)
        .then_some(info)
}

pub(crate) struct ProcessTree {
    roots: usize,
    processes: BTreeMap<libc::pid_t, Identity>,
}
#[derive(Clone, Copy)]
struct Identity {
    pid: libc::pid_t,
    start_seconds: u64,
    start_microseconds: u64,
}
impl Identity {
    fn new(info: &libc::proc_bsdinfo) -> Self {
        Self {
            pid: info.pbi_pid as libc::pid_t,
            start_seconds: info.pbi_start_tvsec,
            start_microseconds: info.pbi_start_tvusec,
        }
    }
    fn matches(self, info: &libc::proc_bsdinfo) -> bool {
        info.pbi_pid == self.pid as u32
            && info.pbi_start_tvsec == self.start_seconds
            && info.pbi_start_tvusec == self.start_microseconds
    }
    fn token(self) -> Option<AuditToken> {
        // Darwin increments pidversion on exec as well as PID reuse. Preserve
        // the birth identity across exec, then obtain the current token with
        // birth checks on both sides. The signal itself validates that token.
        if !self.matches(&process_info(self.pid)?) {
            return None;
        }
        let token = audit_token(self.pid).ok()?;
        self.matches(&process_info(self.pid)?).then_some(token)
    }
    fn signal(self, signal: libc::c_int) -> io::Result<()> {
        self.signal_with(signal, Self::token, AuditToken::signal)
    }
    fn signal_with(
        self,
        signal: libc::c_int,
        mut refresh: impl FnMut(Self) -> Option<AuditToken>,
        mut send: impl FnMut(AuditToken, libc::c_int) -> io::Result<()>,
    ) -> io::Result<()> {
        for _ in 0..8 {
            let Some(token) = refresh(self) else {
                return Ok(());
            };
            match send(token, signal) {
                Ok(()) => return Ok(()),
                Err(error) if matches!(error.raw_os_error(), Some(libc::ESRCH | libc::EINTR)) => {}
                Err(error) => return Err(error),
            }
        }
        Err(io::Error::other("desktop process changed during cleanup"))
    }
}
impl ProcessTree {
    pub(crate) fn new() -> Self {
        Self {
            roots: 0,
            processes: BTreeMap::new(),
        }
    }
    pub(crate) fn root_count(&self) -> usize {
        self.roots
    }
    pub(crate) fn register(
        &mut self,
        token: AuditToken,
        info: &libc::proc_bsdinfo,
    ) -> io::Result<()> {
        // The audit-token syscall checks pidversion in the kernel, rather than
        // sending a signal to a potentially reused numeric PID.
        token.signal(libc::SIGSTOP)?;
        self.processes.insert(token.pid(), Identity::new(info));
        self.roots += 1;
        token.signal(libc::SIGCONT)
    }
    pub(crate) fn discover(&mut self) -> io::Result<()> {
        let known = self.processes.values().copied().collect::<Vec<_>>();
        let mut children = [0_i32; MAX_PROCESSES];
        for parent in known {
            if parent.token().is_none() {
                continue;
            }
            let count = unsafe {
                libc::proc_listpids(
                    PROC_PPID_ONLY,
                    parent.pid as u32,
                    children.as_mut_ptr().cast(),
                    std::mem::size_of_val(&children) as libc::c_int,
                )
            };
            if count < 0 {
                return Err(io::Error::last_os_error());
            }
            let count = count as usize / std::mem::size_of::<libc::pid_t>();
            if count >= children.len() {
                return Err(io::Error::other("desktop command tree limit"));
            }
            for pid in children.iter().take(count).copied().filter(|pid| *pid > 1) {
                let Some(info) = process_info(pid) else {
                    continue;
                };
                if info.pbi_ppid != parent.pid as u32 {
                    continue;
                }
                // A stale numeric PID must not suppress a new owned birth.
                if self
                    .processes
                    .get(&pid)
                    .is_some_and(|known| known.matches(&info))
                {
                    continue;
                }
                let child = Identity::new(&info);
                if child.token().is_none()
                    || parent.token().is_none()
                    || process_info(pid).is_none_or(|current| {
                        current.pbi_ppid != info.pbi_ppid
                            || current.pbi_start_tvsec != info.pbi_start_tvsec
                            || current.pbi_start_tvusec != info.pbi_start_tvusec
                    })
                {
                    continue;
                }
                if self.processes.len() >= MAX_PROCESSES && !self.processes.contains_key(&pid) {
                    return Err(io::Error::other("desktop command tree limit"));
                }
                self.processes.insert(pid, child);
            }
        }
        Ok(())
    }
    pub(crate) fn cleanup(&mut self) -> io::Result<()> {
        // Stop known parents before enumerating so their live children cannot
        // fork again while cleanup expands the bounded tree, including children
        // which created their own sessions/groups.
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut discovery_error = None;
        loop {
            for process in self.processes.values() {
                let _ = process.signal(libc::SIGSTOP);
            }
            let before = self.processes.len();
            if let Err(error) = self.discover() {
                discovery_error = Some(error);
            }
            if self.processes.len() == before || Instant::now() >= deadline {
                break;
            }
        }
        for process in self.processes.values().rev() {
            let _ = process.signal(libc::SIGKILL);
        }
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let alive = self.processes.values().any(|identity| {
                process_info(identity.pid)
                    .is_some_and(|info| identity.matches(&info) && info.pbi_status != 5)
            });
            if !alive {
                // Cancellation keeps the loop alive. Retire completed births,
                // while the separate root counter remains a lifetime bound.
                self.processes.clear();
                return discovery_error.map_or(Ok(()), Err);
            }
            if Instant::now() >= deadline {
                return Err(io::Error::other("desktop command cleanup incomplete"));
            }
            for process in self.processes.values().rev() {
                let _ = process.signal(libc::SIGKILL);
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}
