//! Disposable, credential-free differential probe of the real prototype module.
#[path = "../../../codex-rs/linux-sandbox/src/desktop_process.rs"]
mod desktop_process;
#[path = "../../../codex-rs/cli/src/airs_desktop_linux.rs"]
mod linux_policy;

use std::io;
use std::os::unix::process::CommandExt;
use std::process::Command;

fn expect_errno(name: &str, result: libc::c_long, expected: i32) {
    let actual = io::Error::last_os_error().raw_os_error();
    if result >= 0 {
        if result == 0 {
            unsafe { libc::_exit(0) };
        }
        unsafe { libc::waitpid(result as libc::pid_t, std::ptr::null_mut(), 0) };
    }
    assert_eq!(
        (result, actual),
        (-1, Some(expected)),
        "{name} escaped Desktop restrictions"
    );
}

fn check_boundary() {
    expect_errno(
        "fork",
        unsafe { libc::syscall(libc::SYS_fork) },
        libc::EPERM,
    );
    expect_errno(
        "vfork",
        unsafe { libc::syscall(libc::SYS_vfork) },
        libc::EPERM,
    );
    expect_errno(
        "clone process",
        unsafe { libc::syscall(libc::SYS_clone, libc::SIGCHLD, 0, 0, 0, 0) },
        libc::EPERM,
    );
    expect_errno(
        "clone3",
        unsafe { libc::syscall(libc::SYS_clone3, 0, 0) },
        libc::ENOSYS,
    );
    expect_errno(
        "setsid",
        unsafe { libc::syscall(libc::SYS_setsid) },
        libc::EPERM,
    );
    expect_errno(
        "setpgid",
        unsafe { libc::syscall(libc::SYS_setpgid, 0, 0) },
        libc::EPERM,
    );
    expect_errno(
        "unshare",
        unsafe { libc::syscall(libc::SYS_unshare, libc::CLONE_NEWUSER) },
        libc::EPERM,
    );
    expect_errno(
        "setns",
        unsafe { libc::syscall(libc::SYS_setns, -1, 0) },
        libc::EPERM,
    );
    expect_errno(
        "x32 ABI",
        unsafe { libc::syscall(0x4000_0000 | libc::SYS_getpid) },
        libc::ENOSYS,
    );
    let thread = std::thread::spawn(|| {
        expect_errno(
            "fork from inherited thread",
            unsafe { libc::syscall(libc::SYS_fork) },
            libc::EPERM,
        );
        42
    });
    assert_eq!(
        thread.join().unwrap(),
        42,
        "Ordinary threads must remain available"
    );
    println!("Desktop process boundary passed; thread creation preserved");
}

fn main() {
    assert!(cfg!(all(target_os = "linux", target_arch = "x86_64")));
    match std::env::args().nth(1).as_deref() {
        Some("ordinary-control") => {
            let pid = unsafe { libc::fork() };
            assert!(pid >= 0);
            if pid == 0 {
                unsafe { libc::_exit(0) };
            }
            assert_eq!(unsafe { libc::waitpid(pid, std::ptr::null_mut(), 0) }, pid);
            println!("Ordinary control fork succeeded");
        }
        Some("assert-desktop-without-guard") => check_boundary(),
        Some("guard") => {
            desktop_process::install_command_restrictions().unwrap();
            check_boundary();
        }
        Some("guard-exec") => {
            desktop_process::install_command_restrictions().unwrap();
            let error = Command::new(std::env::current_exe().unwrap())
                .arg("check-inherited")
                .exec();
            panic!("exec replacement failed: {error}");
        }
        Some("check-inherited") => check_boundary(),
        Some("project-policy") => {
            let args: Vec<_> = std::env::args_os().skip(2).collect();
            assert_eq!(args.len(), 3);
            match linux_policy::protect_project(
                std::path::Path::new(&args[0]),
                std::path::Path::new(&args[1]),
                std::path::Path::new(&args[2]),
            ) {
                Ok(()) => println!("Project admitted"),
                Err(_) => std::process::exit(3),
            }
        }
        _ => panic!("Choose a bounded prototype mode"),
    }
}
