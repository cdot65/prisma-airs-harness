use super::*;
use std::process::Command;

#[test]
fn cleanup_refreshes_a_token_after_exec_races_the_signal() {
    let own = process_info(unsafe { libc::getpid() }).unwrap();
    let identity = Identity::new(&own);
    let token = audit_token(identity.pid).unwrap();
    let mut attempts = 0;
    identity
        .signal_with(
            libc::SIGKILL,
            |_| Some(token),
            |_, _| {
                attempts += 1;
                if attempts == 1 {
                    Err(io::Error::from_raw_os_error(libc::ESRCH))
                } else {
                    Ok(())
                }
            },
        )
        .unwrap();
    assert_eq!(attempts, 2);
}

#[test]
fn cleanup_stops_retrying_when_the_original_birth_is_gone() {
    let identity = Identity::new(&process_info(unsafe { libc::getpid() }).unwrap());
    let mut attempts = 0;
    identity
        .signal_with(
            libc::SIGKILL,
            |_| None,
            |_, _| {
                attempts += 1;
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(attempts, 0);
}

#[test]
fn cleanup_reports_exhausted_token_races() {
    let identity = Identity::new(&process_info(unsafe { libc::getpid() }).unwrap());
    let token = audit_token(identity.pid).unwrap();
    let mut attempts = 0;
    assert!(
        identity
            .signal_with(
                libc::SIGKILL,
                |_| Some(token),
                |_, _| {
                    attempts += 1;
                    Err(io::Error::from_raw_os_error(libc::ESRCH))
                }
            )
            .is_err()
    );
    assert_eq!(attempts, 8);
}

#[test]
fn discovery_replaces_stale_birth_for_a_current_owned_descendant() {
    let parent = process_info(unsafe { libc::getpid() }).unwrap();
    let mut child = Command::new("/bin/sleep").arg("30").spawn().unwrap();
    let current = process_info(child.id() as libc::pid_t).unwrap();
    let mut stale = Identity::new(&current);
    stale.start_seconds -= 1;
    let mut tree = ProcessTree::new();
    tree.processes
        .insert(parent.pbi_pid as libc::pid_t, Identity::new(&parent));
    tree.processes.insert(child.id() as libc::pid_t, stale);
    let result = tree.discover();
    let refreshed = tree
        .processes
        .get(&(child.id() as libc::pid_t))
        .unwrap()
        .matches(&current);
    child.kill().unwrap();
    child.wait().unwrap();
    result.unwrap();
    assert!(
        refreshed,
        "cached PID suppressed the current descendant birth"
    );
}

#[test]
fn cancellation_retires_completed_births_but_retains_lifetime_root_count() {
    let mut tree = ProcessTree::new();
    for expected_roots in [1, 2] {
        let mut child = Command::new("/bin/sleep").arg("30").spawn().unwrap();
        let info = process_info(child.id() as libc::pid_t).unwrap();
        tree.register(audit_token(child.id() as libc::pid_t).unwrap(), &info)
            .unwrap();
        tree.cleanup().unwrap();
        child.wait().unwrap();
        assert!(tree.processes.is_empty());
        assert_eq!(tree.root_count(), expected_roots);
    }
}
