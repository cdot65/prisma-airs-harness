//! Bounded private pipe operations shared by both desktop supervision loops.
use std::io;
use std::os::fd::RawFd;

const TIMEOUT_MS: libc::c_int = 3_000;

pub(crate) fn set_cloexec(fd: RawFd) -> io::Result<()> {
    if unsafe { libc::fcntl(fd, libc::F_SETFD, libc::FD_CLOEXEC) } == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

pub(crate) fn set_nonblocking(fd: RawFd) -> io::Result<()> {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags == -1 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

pub(crate) fn write_bounded(fd: RawFd, bytes: &[u8]) -> io::Result<()> {
    let mut poll = libc::pollfd {
        fd,
        events: libc::POLLOUT,
        revents: 0,
    };
    if unsafe { libc::poll(&raw mut poll, 1, TIMEOUT_MS) } <= 0
        || unsafe { libc::write(fd, bytes.as_ptr().cast(), bytes.len()) } != bytes.len() as isize
    {
        return Err(unavailable());
    }
    Ok(())
}

pub(crate) fn read_exact_bounded(fd: RawFd, bytes: &mut [u8]) -> io::Result<()> {
    read_exact_with_timeout(fd, bytes, TIMEOUT_MS)
}

pub(crate) fn read_exact_with_timeout(
    fd: RawFd,
    bytes: &mut [u8],
    timeout_ms: libc::c_int,
) -> io::Result<()> {
    let mut poll = libc::pollfd {
        fd,
        events: libc::POLLIN,
        revents: 0,
    };
    if unsafe { libc::poll(&raw mut poll, 1, timeout_ms) } <= 0 {
        return Err(unavailable());
    }
    if unsafe { libc::read(fd, bytes.as_mut_ptr().cast(), bytes.len()) } != bytes.len() as isize {
        return Err(unavailable());
    }
    Ok(())
}

fn unavailable() -> io::Error {
    io::Error::other("desktop command supervision unavailable")
}
