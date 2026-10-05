use super::*;
use std::collections::HashMap;
use std::fs;
use std::os::unix::process::ExitStatusExt;

#[test]
fn hidden_supervisor_rejects_an_unrelated_launcher() {
    assert!(run_desktop_supervisor().is_err());
}

#[test]
#[ignore = "private subprocess entry point"]
fn broker_fixture() {
    run_desktop_supervisor().unwrap();
}

#[test]
#[ignore = "private subprocess entry point"]
fn mismatched_ack_fixture() {
    assert_eq!(unsafe { libc::write(1, [READY].as_ptr().cast(), 1) }, 1);
    let mut registration = [0_u8; std::mem::size_of::<AuditToken>()];
    read_exact_bounded(0, &mut registration).unwrap();
    registration[31] ^= 1;
    assert_eq!(
        unsafe { libc::write(1, registration.as_ptr().cast(), registration.len()) },
        registration.len() as isize
    );
}

#[test]
#[ignore = "private subprocess entry point"]
fn mismatch_worker_fixture() {
    let root = tempfile::tempdir().unwrap();
    let mut broker = Command::new(std::env::current_exe().unwrap());
    broker.args([
        "--exact",
        "desktop_supervisor::tests::mismatched_ack_fixture",
        "--ignored",
        "--nocapture",
        "--test-threads=1",
    ]);
    let supervisor = start(broker, ReadyProtocol::TestHarness).unwrap();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let marker = root.path().join("must-not-execute");
    let result = runtime.block_on(crate::spawn_pipe_process(
        "/usr/bin/touch",
        &[marker.to_string_lossy().into_owned()],
        root.path(),
        &HashMap::new(),
        &None,
        &[],
    ));
    assert!(result.is_err());
    assert!(!marker.exists());
    drop(supervisor);
}

#[test]
fn mismatched_registration_ack_cannot_execute_a_command() {
    let status = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "desktop_supervisor::tests::mismatch_worker_fixture",
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
#[ignore = "private subprocess entry point"]
fn worker_fixture() {
    let root = std::env::var("DESKTOP_TEST_ROOT").unwrap();
    let backend = std::env::var("DESKTOP_TEST_BACKEND").unwrap();
    let ending = std::env::var("DESKTOP_TEST_ENDING").unwrap();
    let detached = std::env::var("DESKTOP_TEST_DETACHED").unwrap() == "yes";
    let executable = std::env::current_exe().unwrap();
    let mut broker = Command::new(executable);
    broker.args([
        "--exact",
        "desktop_supervisor::tests::broker_fixture",
        "--ignored",
        "--nocapture",
        "--test-threads=1",
    ]);
    let supervisor = start(broker, ReadyProtocol::TestHarness).unwrap();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        // Side effects are released only after the requested stop. A slow
        // scheduler cannot start the marker timer before cleanup is tested.
        let script = format!(
            r#"import os,time,pathlib
root=pathlib.Path({root:?})
def after_stop(name):
 while not (root/'release-after-stop').exists():
  time.sleep(0.01)
 time.sleep(0.1)
 (root/name).write_text('escaped')
p=os.fork()
if p==0:
 {}
 (root/'grandchild-ready').write_text(str(os.getpid()))
 after_stop('grandchild-marker')
 os._exit(0)
(root/'child-ready').write_text(str(os.getpid()))
after_stop('child-marker')
time.sleep(30)
"#,
            if detached { "os.setsid()" } else { "pass" }
        );
        let args = vec!["-c".into(), script];
        let process = if backend == "pty" {
            crate::spawn_pty_process(
                "/usr/bin/python3",
                &args,
                Path::new(&root),
                &HashMap::new(),
                &None,
                crate::TerminalSize::default(),
                &[],
            )
            .await
            .unwrap()
        } else {
            crate::spawn_pipe_process(
                "/usr/bin/python3",
                &args,
                Path::new(&root),
                &HashMap::new(),
                &None,
                &[],
            )
            .await
            .unwrap()
        };
        // Deliberately omit the ordinary ProcessHandle destructor so only the
        // independent supervisor can establish cleanup in these fixtures.
        std::mem::forget(process);
    });
    wait_file(&Path::new(&root).join("grandchild-ready"));
    std::thread::sleep(Duration::from_millis(100));
    fs::write(Path::new(&root).join("worker-ready"), "ready").unwrap();
    if ending == "drop" {
        drop(supervisor);
        assert!(
            runtime
                .block_on(crate::spawn_pipe_process(
                    "/bin/true",
                    &[],
                    Path::new(&root),
                    &HashMap::new(),
                    &None,
                    &[]
                ))
                .is_err()
        );
    } else if ending == "broker-death" {
        assert_eq!(
            unsafe { libc::kill(supervisor.child.id() as libc::pid_t, libc::SIGKILL) },
            0
        );
        let deadline = Instant::now() + Duration::from_secs(3);
        while supervisor.check_health().is_ok() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(supervisor.check_health().is_err());
        assert!(
            runtime
                .block_on(crate::spawn_pipe_process(
                    "/bin/true",
                    &[],
                    Path::new(&root),
                    &HashMap::new(),
                    &None,
                    &[]
                ))
                .is_err()
        );
        drop(supervisor);
    } else if ending == "cancel" {
        let stopped = ["child-ready", "grandchild-ready"].map(|name| {
            let path = Path::new(&root).join(name);
            wait_file(&path);
            let pid = fs::read_to_string(path).unwrap().parse().unwrap();
            process_info(pid).unwrap()
        });
        let broker = process_info(supervisor.child.id() as libc::pid_t).unwrap();
        cancel_desktop_commands().unwrap();
        for previous in stopped {
            assert!(
                process_info(previous.pbi_pid as libc::pid_t).is_none_or(|current| {
                    current.pbi_start_tvsec != previous.pbi_start_tvsec
                        || current.pbi_start_tvusec != previous.pbi_start_tvusec
                        || current.pbi_status == 5
                }),
                "cancel must retire the owned birth identity before acknowledging"
            );
        }
        // A second empty cancellation has a distinct control sequence and must
        // leave both loops healthy instead of replaying a previous ACK.
        cancel_desktop_commands().unwrap();
        supervisor.check_health().unwrap();
        assert!(
            process_info(broker.pbi_pid as libc::pid_t).is_some_and(|current| {
                current.pbi_start_tvsec == broker.pbi_start_tvsec
                    && current.pbi_start_tvusec == broker.pbi_start_tvusec
                    && current.pbi_status != 5
            })
        );
        runtime.block_on(async {
            let marker = Path::new(&root).join("fresh-command-marker");
            let args = vec![marker.to_string_lossy().into_owned()];
            let process = if backend == "pty" {
                crate::spawn_pty_process(
                    "/usr/bin/touch",
                    &args,
                    Path::new(&root),
                    &HashMap::new(),
                    &None,
                    crate::TerminalSize::default(),
                    &[],
                )
                .await
                .unwrap()
            } else {
                crate::spawn_pipe_process(
                    "/usr/bin/touch",
                    &args,
                    Path::new(&root),
                    &HashMap::new(),
                    &None,
                    &[],
                )
                .await
                .unwrap()
            };
            assert_eq!(
                tokio::time::timeout(Duration::from_secs(3), process.exit_rx)
                    .await
                    .unwrap()
                    .unwrap(),
                0
            );
            assert!(marker.exists());
        });
        supervisor.check_health().unwrap();
        // Prove cancellation itself before Drop can perform final retirement.
        fs::write(Path::new(&root).join("release-after-stop"), "release").unwrap();
        std::thread::sleep(Duration::from_millis(300));
        assert!(!Path::new(&root).join("child-marker").exists());
        assert!(!Path::new(&root).join("grandchild-marker").exists());
        supervisor.check_health().unwrap();
        drop(supervisor);
    } else {
        std::thread::sleep(Duration::from_secs(30));
        drop(supervisor);
    }
}

fn wait_file(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !path.exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(
        path.exists(),
        "fixture readiness missing: {}",
        path.display()
    );
}
enum End {
    HardDeath,
    Drop,
    BrokerDeath,
    Cancel,
}
enum Backend {
    Pipe,
    Pty,
}
enum Descendant {
    Ordinary,
    SeparateSession,
}
fn check_cleanup(backend: Backend, end: End, descendant: Descendant) {
    let root = tempfile::tempdir().unwrap();
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--exact",
            "desktop_supervisor::tests::worker_fixture",
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ])
        .env("DESKTOP_TEST_ROOT", root.path())
        .env(
            "DESKTOP_TEST_BACKEND",
            match backend {
                Backend::Pipe => "pipe",
                Backend::Pty => "pty",
            },
        )
        .env(
            "DESKTOP_TEST_ENDING",
            match end {
                End::HardDeath => "kill",
                End::Drop => "drop",
                End::BrokerDeath => "broker-death",
                End::Cancel => "cancel",
            },
        )
        .env(
            "DESKTOP_TEST_DETACHED",
            match descendant {
                Descendant::Ordinary => "no",
                Descendant::SeparateSession => "yes",
            },
        )
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    unsafe {
        command.pre_exec(crate::process_group::detach_from_tty);
    }
    let mut worker = command.spawn().unwrap();
    wait_file(&root.path().join("worker-ready"));
    if matches!(end, End::HardDeath) {
        assert_eq!(
            unsafe { libc::killpg(worker.id() as i32, libc::SIGKILL) },
            0
        );
        assert_eq!(worker.wait().unwrap().signal(), Some(libc::SIGKILL));
    } else {
        assert!(worker.wait().unwrap().success());
    }
    fs::write(root.path().join("release-after-stop"), "release").unwrap();
    std::thread::sleep(Duration::from_millis(300));
    assert!(!root.path().join("child-marker").exists());
    assert!(!root.path().join("grandchild-marker").exists());
    if matches!(end, End::Cancel) {
        assert!(root.path().join("fresh-command-marker").exists());
    }
}
#[test]
fn pipe_hard_parent_death_cleans_child_and_grandchild() {
    check_cleanup(Backend::Pipe, End::HardDeath, Descendant::Ordinary);
}
#[test]
fn raw_pty_hard_parent_death_cleans_child_and_grandchild() {
    check_cleanup(Backend::Pty, End::HardDeath, Descendant::Ordinary);
}
#[test]
fn pipe_supervisor_drop_cleans_separate_session_descendant() {
    check_cleanup(Backend::Pipe, End::Drop, Descendant::SeparateSession);
}
#[test]
fn raw_pty_supervisor_drop_cleans_separate_session_descendant() {
    check_cleanup(Backend::Pty, End::Drop, Descendant::SeparateSession);
}
#[test]
fn pipe_hard_parent_death_cleans_separate_session_descendant() {
    check_cleanup(Backend::Pipe, End::HardDeath, Descendant::SeparateSession);
}
#[test]
fn raw_pty_hard_parent_death_cleans_separate_session_descendant() {
    check_cleanup(Backend::Pty, End::HardDeath, Descendant::SeparateSession);
}

#[test]
fn pipe_broker_death_cleans_separate_session_descendant() {
    check_cleanup(Backend::Pipe, End::BrokerDeath, Descendant::SeparateSession);
}

#[test]
fn raw_pty_broker_death_cleans_separate_session_descendant() {
    check_cleanup(Backend::Pty, End::BrokerDeath, Descendant::SeparateSession);
}

#[test]
fn pipe_cancel_cleans_separate_session_descendant_and_allows_new_command() {
    check_cleanup(Backend::Pipe, End::Cancel, Descendant::SeparateSession);
}

#[test]
fn raw_pty_cancel_cleans_separate_session_descendant_and_allows_new_command() {
    check_cleanup(Backend::Pty, End::Cancel, Descendant::SeparateSession);
}

#[test]
#[ignore = "private subprocess entry point"]
fn stale_control_ack_fixture() {
    assert_eq!(unsafe { libc::write(1, [READY].as_ptr().cast(), 1) }, 1);
    let mut record = [0_u8; std::mem::size_of::<AuditToken>()];
    read_exact_bounded(0, &mut record).unwrap();
    // The correct tag with a previous sequence cannot acknowledge this call.
    record[24..].copy_from_slice(&0_u64.to_le_bytes());
    write_bounded(1, &record).unwrap();
}

#[test]
#[ignore = "private subprocess entry point"]
fn stale_control_worker_fixture() {
    let mut broker = Command::new(std::env::current_exe().unwrap());
    broker.args([
        "--exact",
        "desktop_supervisor::tests::stale_control_ack_fixture",
        "--ignored",
        "--nocapture",
        "--test-threads=1",
    ]);
    let supervisor = start(broker, ReadyProtocol::TestHarness).unwrap();
    assert!(cancel_desktop_commands().is_err());
    assert!(cancel_desktop_commands().is_err());
    assert!(supervisor.check_health().is_err());
    assert!(prepare_spawn().is_err());
    drop(supervisor);
}

#[test]
fn stale_control_ack_fails_closed_for_later_controls_and_commands() {
    let status = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "desktop_supervisor::tests::stale_control_worker_fixture",
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
#[ignore = "private subprocess entry point"]
fn invalid_control_worker_fixture() {
    let mut broker = Command::new(std::env::current_exe().unwrap());
    broker.args([
        "--exact",
        "desktop_supervisor::tests::broker_fixture",
        "--ignored",
        "--nocapture",
        "--test-threads=1",
    ]);
    let mut supervisor = start(broker, ReadyProtocol::TestHarness).unwrap();
    let registration = match *SPAWN.lock().unwrap() {
        State::Active(registration) => registration,
        State::Ordinary | State::Unavailable => panic!("fixture supervisor missing"),
    };
    let mut record = [0_u8; std::mem::size_of::<AuditToken>()];
    record[..16].copy_from_slice(CANCEL_TAG);
    record[24..].copy_from_slice(&1_u64.to_le_bytes());
    if std::env::var("DESKTOP_TEST_CONTROL").unwrap() == "stale" {
        write_bounded(registration.input, &record).unwrap();
        let mut ack = [0_u8; std::mem::size_of::<AuditToken>()];
        read_exact_bounded(registration.output, &mut ack).unwrap();
        assert_eq!(ack, record);
    } else {
        record[0] ^= 1;
    }
    write_bounded(registration.input, &record).unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while supervisor.check_health().is_ok() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(supervisor.check_health().is_err());
    assert!(prepare_spawn().is_err());
    assert!(!supervisor.child.wait().unwrap().success());
    drop(supervisor);
}

#[test]
fn malformed_or_repeated_control_records_retire_supervision() {
    for control in ["malformed", "stale"] {
        let status = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "desktop_supervisor::tests::invalid_control_worker_fixture",
                "--ignored",
                "--nocapture",
                "--test-threads=1",
            ])
            .env("DESKTOP_TEST_CONTROL", control)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap();
        assert!(status.success(), "control case: {control}");
    }
}

#[test]
#[ignore = "private subprocess entry point"]
fn zero_root_worker_fixture() {
    let mut broker = Command::new(std::env::current_exe().unwrap());
    broker.args([
        "--exact",
        "desktop_supervisor::tests::broker_fixture",
        "--ignored",
        "--nocapture",
        "--test-threads=1",
    ]);
    let supervisor = start(broker, ReadyProtocol::TestHarness).unwrap();
    let info = process_info(supervisor.child.id() as libc::pid_t).unwrap();
    drop(supervisor);
    assert!(
        process_info(info.pbi_pid as libc::pid_t).is_none_or(|current| {
            current.pbi_start_tvsec != info.pbi_start_tvsec
                || current.pbi_start_tvusec != info.pbi_start_tvusec
        })
    );
}

#[test]
fn private_zero_root_eof_finishes_and_reaps_the_supervisor() {
    let status = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "desktop_supervisor::tests::zero_root_worker_fixture",
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();
    assert!(status.success());
}
