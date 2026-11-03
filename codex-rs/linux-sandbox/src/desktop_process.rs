//! Strict Desktop command process boundary; selected only by the owned helper.
//!
//! Install only in the final command child, after the existing Bubblewrap
//! filesystem/PID/network namespaces, capability drop and restricted-network
//! filter. This is not a filesystem/network sandbox or a lifetime supervisor.
//! Every installation failure must retire the command before user code runs.
use seccompiler::BpfProgram;
use seccompiler::SeccompAction;
use seccompiler::SeccompCmpArgLen;
use seccompiler::SeccompCmpOp;
use seccompiler::SeccompCondition;
use seccompiler::SeccompFilter;
use seccompiler::SeccompRule;
use seccompiler::TargetArch;
use seccompiler::apply_filter_all_threads;
use seccompiler::sock_filter;
use std::collections::BTreeMap;
use std::io;

/// Admit threads and exec replacement, never another process or session escape.
///
/// This implementation intentionally supports Linux AMD64 only. Ordinary
/// Harness execution is unchanged; the strict Desktop inner child selects it.
pub fn install_command_restrictions() -> io::Result<()> {
    if !cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Desktop process prototype requires Linux AMD64",
        ));
    }
    let mut rules = BTreeMap::new();
    for syscall in [
        libc::SYS_fork,
        libc::SYS_vfork,
        libc::SYS_setsid,
        libc::SYS_setpgid,
        libc::SYS_unshare,
        libc::SYS_setns,
    ] {
        rules.insert(syscall, vec![]);
    }
    // CLONE_THREAD is required: a successful clone remains inside this TGID.
    // The kernel also requires CLONE_VM/CLONE_SIGHAND for CLONE_THREAD.
    rules.insert(
        libc::SYS_clone,
        vec![
            SeccompRule::new(vec![
                SeccompCondition::new(
                    0,
                    SeccompCmpArgLen::Qword,
                    SeccompCmpOp::MaskedEq(libc::CLONE_THREAD as u64),
                    0,
                )
                .map_err(io::Error::other)?,
            ])
            .map_err(io::Error::other)?,
        ],
    );
    let process_filter: BpfProgram = SeccompFilter::new(
        rules,
        SeccompAction::Allow,
        SeccompAction::Errno(libc::EPERM as u32),
        TargetArch::x86_64,
    )
    .map_err(io::Error::other)?
    .try_into()
    .map_err(io::Error::other)?;
    // clone3's flags live behind a userspace pointer, which seccomp cannot
    // inspect. ENOSYS preserves libc's clone-based pthread fallback.
    let clone3_filter: BpfProgram = SeccompFilter::new(
        BTreeMap::from([(libc::SYS_clone3, vec![])]),
        SeccompAction::Allow,
        SeccompAction::Errno(libc::ENOSYS as u32),
        TargetArch::x86_64,
    )
    .map_err(io::Error::other)?
    .try_into()
    .map_err(io::Error::other)?;
    // seccompiler 0.5 validates audit architecture, but x32 shares x86_64's
    // audit value. Refuse the entire x32 syscall range before matching numbers.
    let abi_filter = vec![
        sock_filter {
            code: 0x20,
            jt: 0,
            jf: 0,
            k: 4,
        },
        sock_filter {
            code: 0x15,
            jt: 1,
            jf: 0,
            k: 0xc000_003e,
        },
        sock_filter {
            code: 0x06,
            jt: 0,
            jf: 0,
            k: libc::SECCOMP_RET_KILL_PROCESS,
        },
        sock_filter {
            code: 0x20,
            jt: 0,
            jf: 0,
            k: 0,
        },
        sock_filter {
            code: 0x35,
            jt: 0,
            jf: 1,
            k: 0x4000_0000,
        },
        sock_filter {
            code: 0x06,
            jt: 0,
            jf: 0,
            k: libc::SECCOMP_RET_ERRNO | libc::ENOSYS as u32,
        },
        sock_filter {
            code: 0x06,
            jt: 0,
            jf: 0,
            k: libc::SECCOMP_RET_ALLOW,
        },
    ];
    for filter in [&abi_filter, &clone3_filter, &process_filter] {
        apply_filter_all_threads(filter).map_err(io::Error::other)?;
    }
    Ok(())
}
