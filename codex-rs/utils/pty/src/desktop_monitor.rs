//! Secret-free engine-side fallback if the independent helper itself dies.
use std::fs::File;
use std::io;
use std::os::fd::AsRawFd;
use std::os::fd::FromRawFd;
use std::os::fd::RawFd;
use std::process::Child;
use std::thread::JoinHandle;
use std::time::Duration;
use std::time::Instant;

use crate::desktop_process_tree::process_info;
use crate::desktop_supervisor::mark_unavailable;
use crate::desktop_supervisor::run_registration_loop;
use crate::desktop_supervisor_io::set_cloexec;
use crate::desktop_supervisor_io::set_nonblocking;

pub(crate) struct Monitor {
    input: Option<File>,
    output: Option<File>,
    worker: Option<JoinHandle<()>>,
}
impl Monitor {
    pub(crate) fn start(broker: &Child) -> io::Result<Self> {
        let broker = process_info(broker.id() as libc::pid_t)
            .ok_or_else(|| io::Error::other("desktop supervisor identity unavailable"))?;
        let (reader, input) = private_pipe()?;
        let (output, writer) = private_pipe()?;
        set_nonblocking(input.as_raw_fd())?;
        set_nonblocking(writer.as_raw_fd())?;
        let owner = unsafe { libc::getpid() };
        let worker = std::thread::Builder::new()
            .name("desktop-command-watch".into())
            .spawn(move || {
                // Only pipe handles and process birth metadata cross this seam.
                let _ =
                    run_registration_loop(reader.as_raw_fd(), writer.as_raw_fd(), owner, || {
                        process_info(broker.pbi_pid as libc::pid_t).is_some_and(|current| {
                            current.pbi_start_tvsec == broker.pbi_start_tvsec
                                && current.pbi_start_tvusec == broker.pbi_start_tvusec
                                && current.pbi_status != 5 // SZOMB, sys/proc.h
                        })
                    });
                // This atomic update never waits on the parent's spawn mutex.
                mark_unavailable();
            })?;
        Ok(Self {
            input: Some(input),
            output: Some(output),
            worker: Some(worker),
        })
    }
    pub(crate) fn descriptors(&self) -> io::Result<(RawFd, RawFd)> {
        let (Some(input), Some(output)) = (&self.input, &self.output) else {
            return Err(io::Error::other("desktop command monitor unavailable"));
        };
        Ok((input.as_raw_fd(), output.as_raw_fd()))
    }
}
impl Drop for Monitor {
    fn drop(&mut self) {
        self.input.take();
        self.output.take();
        if let Some(worker) = self.worker.take() {
            let deadline = Instant::now() + Duration::from_secs(5);
            while !worker.is_finished() && Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(10));
            }
            if worker.is_finished() {
                let _ = worker.join();
            }
        }
    }
}
fn private_pipe() -> io::Result<(File, File)> {
    let mut fds = [-1; 2];
    if unsafe { libc::pipe(fds.as_mut_ptr()) } == -1 {
        return Err(io::Error::last_os_error());
    }
    let reader = unsafe { File::from_raw_fd(fds[0]) };
    let writer = unsafe { File::from_raw_fd(fds[1]) };
    set_cloexec(reader.as_raw_fd())?;
    set_cloexec(writer.as_raw_fd())?;
    Ok((reader, writer))
}
