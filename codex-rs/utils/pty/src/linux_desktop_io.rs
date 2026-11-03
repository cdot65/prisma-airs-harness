//! Fixed-size, bounded private supervisor records with kernel-owned pidfds.
use std::io;
use std::os::fd::AsRawFd;
use std::os::fd::FromRawFd;
use std::os::fd::OwnedFd;
use std::os::fd::RawFd;

pub(super) const TIMEOUT_MS: i32 = 3_000;
pub(super) type Record = [u64; 2];

pub(super) fn pair() -> io::Result<(OwnedFd, OwnedFd)> {
    let mut fds = [-1; 2];
    // SAFETY: socketpair initializes exactly two new descriptors.
    if unsafe {
        libc::socketpair(
            libc::AF_UNIX,
            libc::SOCK_SEQPACKET | libc::SOCK_CLOEXEC,
            0,
            fds.as_mut_ptr(),
        )
    } != 0
    {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: both returned descriptors are newly owned.
    Ok(unsafe { (OwnedFd::from_raw_fd(fds[0]), OwnedFd::from_raw_fd(fds[1])) })
}

pub(super) fn poll(fd: RawFd, timeout: i32) -> io::Result<bool> {
    let mut event = libc::pollfd {
        fd,
        events: libc::POLLIN,
        revents: 0,
    };
    // SAFETY: poll receives one initialized descriptor record.
    match unsafe { libc::poll(&raw mut event, 1, timeout) } {
        -1 => Err(io::Error::last_os_error()),
        0 => Ok(false),
        _ if event.revents & libc::POLLNVAL != 0 => Err(unavailable()),
        _ => Ok(true),
    }
}

pub(super) fn pidfd(pid: libc::pid_t) -> io::Result<OwnedFd> {
    // SAFETY: pidfd_open returns a new CLOEXEC descriptor and has no pointers.
    let fd = unsafe { libc::syscall(libc::SYS_pidfd_open, pid, 0) };
    if fd == -1 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: the successful syscall returned a new owned descriptor.
    Ok(unsafe { OwnedFd::from_raw_fd(fd as RawFd) })
}

pub(super) fn signal(fd: &OwnedFd, signal: i32) -> io::Result<()> {
    // SAFETY: the pidfd identifies a retained process birth, never a PID lookup.
    if unsafe {
        libc::syscall(
            libc::SYS_pidfd_send_signal,
            fd.as_raw_fd(),
            signal,
            std::ptr::null::<libc::siginfo_t>(),
            0,
        )
    } == -1
    {
        let error = io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::ESRCH) {
            return Err(error);
        }
    }
    Ok(())
}

pub(super) fn send(socket: RawFd, record: Record, fd: Option<RawFd>) -> io::Result<()> {
    // Aligned storage fits one SCM_RIGHTS descriptor on supported Linux ABIs.
    let mut control = [0_usize; 8];
    let mut vector = libc::iovec {
        iov_base: record.as_ptr().cast_mut().cast(),
        iov_len: std::mem::size_of::<Record>(),
    };
    // SAFETY: zeroed msghdr has null optional fields; the iovec is valid.
    let mut message: libc::msghdr = unsafe { std::mem::zeroed() };
    message.msg_iov = &raw mut vector;
    message.msg_iovlen = 1;
    if let Some(fd) = fd {
        message.msg_control = control.as_mut_ptr().cast();
        // SAFETY: this control buffer is sufficiently sized and aligned.
        unsafe {
            message.msg_controllen = libc::CMSG_SPACE(std::mem::size_of::<RawFd>() as u32) as usize;
            let header = libc::CMSG_FIRSTHDR(&message);
            (*header).cmsg_level = libc::SOL_SOCKET;
            (*header).cmsg_type = libc::SCM_RIGHTS;
            (*header).cmsg_len = libc::CMSG_LEN(std::mem::size_of::<RawFd>() as u32) as usize;
            libc::CMSG_DATA(header).cast::<RawFd>().write(fd);
        }
    }
    let mut writable = libc::pollfd {
        fd: socket,
        events: libc::POLLOUT,
        revents: 0,
    };
    // SAFETY: the bounded syscall reads only this stack record and control data.
    if unsafe { libc::poll(&raw mut writable, 1, TIMEOUT_MS) } <= 0
        || unsafe { libc::sendmsg(socket, &message, libc::MSG_NOSIGNAL | libc::MSG_DONTWAIT) }
            != std::mem::size_of::<Record>() as isize
    {
        return Err(unavailable());
    }
    Ok(())
}

pub(super) fn receive(socket: RawFd) -> io::Result<Option<(Record, Option<OwnedFd>)>> {
    if !poll(socket, TIMEOUT_MS)? {
        return Err(unavailable());
    }
    let mut record = [0; 2];
    let mut control = [0_usize; 8];
    let mut vector = libc::iovec {
        iov_base: record.as_mut_ptr().cast(),
        iov_len: std::mem::size_of::<Record>(),
    };
    // SAFETY: all pointers describe writable, stack-owned buffers.
    let mut message: libc::msghdr = unsafe { std::mem::zeroed() };
    message.msg_iov = &raw mut vector;
    message.msg_iovlen = 1;
    message.msg_control = control.as_mut_ptr().cast();
    message.msg_controllen = std::mem::size_of_val(&control);
    // MSG_CMSG_CLOEXEC also closes any received capability on a later exec.
    let length = unsafe {
        libc::recvmsg(
            socket,
            &raw mut message,
            libc::MSG_CMSG_CLOEXEC | libc::MSG_DONTWAIT,
        )
    };
    if length == -1 {
        return Err(io::Error::last_os_error());
    }
    let mut descriptor = None;
    let mut count = 0;
    // SAFETY: recvmsg initialized the bounded ancillary buffer. Take ownership
    // even of rejected extras so malformed input cannot leak a descriptor.
    unsafe {
        let mut header = libc::CMSG_FIRSTHDR(&message);
        while !header.is_null() {
            if (*header).cmsg_level == libc::SOL_SOCKET && (*header).cmsg_type == libc::SCM_RIGHTS {
                let bytes = (*header)
                    .cmsg_len
                    .saturating_sub(libc::CMSG_LEN(0) as usize);
                for index in 0..bytes / std::mem::size_of::<RawFd>() {
                    let received = OwnedFd::from_raw_fd(
                        libc::CMSG_DATA(header).cast::<RawFd>().add(index).read(),
                    );
                    count += 1;
                    if descriptor.is_none() {
                        descriptor = Some(received);
                    }
                }
            }
            header = libc::CMSG_NXTHDR(&message, header);
        }
    }
    if length == 0 && descriptor.is_none() {
        return Ok(None);
    }
    if length != std::mem::size_of::<Record>() as isize
        || message.msg_flags & (libc::MSG_TRUNC | libc::MSG_CTRUNC) != 0
        || count > 1
    {
        return Err(unavailable());
    }
    Ok(Some((record, descriptor)))
}

pub(super) fn unavailable() -> io::Error {
    io::Error::from_raw_os_error(libc::EIO)
}
