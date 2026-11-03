use super::*;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn fixture(name: &str) -> Command {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command.args([
        "--exact",
        &format!("desktop_supervisor::tests::{name}"),
        "--ignored",
        "--nocapture",
        "--test-threads=1",
    ]);
    command
}

fn wait_file(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !path.exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(path.exists(), "fixture did not reach {}", path.display());
}

#[test]
fn desktop_bootstrap_requires_an_active_supervisor() {
    assert!(require_desktop_command_supervision().is_err());
    assert!(run_desktop_supervisor().is_err());
}

#[test]
fn descriptor_transfer_retains_the_exact_process_birth() {
    let (sender, receiver) = channel::pair().unwrap();
    let own = channel::pidfd(unsafe { libc::getpid() }).unwrap();
    channel::send(sender.as_raw_fd(), [REGISTER, 7], Some(own.as_raw_fd())).unwrap();
    let (record, received) = channel::receive(receiver.as_raw_fd()).unwrap().unwrap();
    assert_eq!(record, [REGISTER, 7]);
    let received = received.unwrap();
    assert!(!channel::poll(received.as_raw_fd(), 0).unwrap());
    channel::signal(&received, 0).unwrap();
    assert_ne!(
        unsafe { libc::fcntl(received.as_raw_fd(), libc::F_GETFD) } & libc::FD_CLOEXEC,
        0
    );
}

#[test]
fn a_retired_birth_handle_cannot_target_a_later_child() {
    let mut previous = Command::new("/bin/true").spawn().unwrap();
    let retired = DesktopCommand(Arc::new(
        channel::pidfd(previous.id() as libc::pid_t).unwrap(),
    ));
    assert!(previous.wait().unwrap().success());
    let mut later = Command::new("/bin/sleep").arg("60").spawn().unwrap();
    let later_birth = channel::pidfd(later.id() as libc::pid_t).unwrap();
    retired.signal(libc::SIGKILL).unwrap();
    assert!(!channel::poll(later_birth.as_raw_fd(), 0).unwrap());
    later.kill().unwrap();
    later.wait().unwrap();
}

#[test]
fn cleanup_attempts_every_owned_root_after_a_descriptor_error() {
    let invalid: OwnedFd = fs::File::open("/dev/null").unwrap().into();
    let mut child = Command::new("/bin/sleep").arg("60").spawn().unwrap();
    let birth = channel::pidfd(child.id() as libc::pid_t).unwrap();
    assert!(kill_roots([&invalid, &birth].into_iter()).is_err());
    assert!(channel::poll(birth.as_raw_fd(), 0).unwrap());
    assert!(!child.wait().unwrap().success());
}

#[test]
#[ignore = "private subprocess entry point"]
fn broker_fixture() {
    run_desktop_supervisor().unwrap();
}

#[test]
#[ignore = "private subprocess entry point"]
fn bad_ack_broker_fixture() {
    channel::send(0, READY, None).unwrap();
    let (record, pidfd) = channel::receive(0).unwrap().unwrap();
    let pidfd = pidfd.unwrap();
    channel::send(0, [record[0], record[1] + 1], None).unwrap();
    assert!(channel::poll(pidfd.as_raw_fd(), 3_000).unwrap());
}

#[test]
#[ignore = "private subprocess entry point"]
fn bad_ack_worker_fixture() {
    let root = tempfile::tempdir().unwrap();
    let supervisor = start(fixture("bad_ack_broker_fixture")).unwrap();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let marker = root.path().join("must-not-execute");
    assert!(
        runtime
            .block_on(crate::spawn_pipe_process(
                "/usr/bin/touch",
                &[marker.to_string_lossy().into_owned()],
                root.path(),
                &HashMap::new(),
                &None,
                &[],
            ))
            .is_err()
    );
    assert!(!marker.exists());
    assert!(require_desktop_command_supervision().is_err());
    drop(supervisor);
}

#[test]
fn mismatched_ack_cannot_release_user_code() {
    assert!(
        fixture("bad_ack_worker_fixture")
            .stdout(Stdio::null())
            .status()
            .unwrap()
            .success()
    );
}

#[test]
#[ignore = "private subprocess entry point"]
fn worker_fixture() {
    let root = PathBuf::from(std::env::var_os("AIRS_PTY_FIXTURE_ROOT").unwrap());
    let backend = std::env::var("AIRS_PTY_FIXTURE_BACKEND").unwrap();
    let ending = std::env::var("AIRS_PTY_FIXTURE_ENDING").unwrap();
    let supervisor = start(fixture("broker_fixture")).unwrap();
    require_desktop_command_supervision().unwrap();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    if ending == "spawn-error" {
        let result = runtime.block_on(async {
            if backend == "pty" {
                crate::spawn_pty_process(
                    "/this-fixture-program-does-not-exist",
                    &[],
                    &root,
                    &HashMap::new(),
                    &None,
                    crate::TerminalSize::default(),
                    &[],
                )
                .await
            } else {
                crate::spawn_pipe_process(
                    "/this-fixture-program-does-not-exist",
                    &[],
                    &root,
                    &HashMap::new(),
                    &None,
                    &[],
                )
                .await
            }
        });
        assert!(result.is_err());
        assert!(require_desktop_command_supervision().is_err());
        return;
    }
    // Deliberately clear parent-death signaling and ignore TERM in the fixture:
    // an independent pidfd path must establish cleanup, including exec and threads.
    let script = format!(
        r#"import ctypes,os,pathlib,signal,stat,threading,time
root=pathlib.Path({root:?})
for fd in os.listdir('/proc/self/fd'):
 try: assert not stat.S_ISSOCK(os.fstat(int(fd)).st_mode), 'inherited supervisor capability'
 except OSError: pass
ctypes.CDLL(None).prctl(1,0,0,0,0)
signal.signal(signal.SIGTERM,signal.SIG_IGN)
threading.Thread(target=lambda:time.sleep(60),daemon=True).start()
(root/'root-ready').write_text(str(os.getpid()))
while not (root/'release-after-stop').exists(): time.sleep(0.01)
(root/'escaped-marker').write_text('escaped')
"#
    );
    let args = vec![
        "-c".into(),
        "import os,sys; os.execv(sys.executable,[sys.executable,'-c',sys.argv[1]])".into(),
        script,
    ];
    let process = runtime.block_on(async {
        if backend == "pty" {
            crate::spawn_pty_process(
                "/usr/bin/python3",
                &args,
                &root,
                &HashMap::new(),
                &None,
                crate::TerminalSize::default(),
                &[],
            )
            .await
        } else {
            crate::spawn_pipe_process(
                "/usr/bin/python3",
                &args,
                &root,
                &HashMap::new(),
                &None,
                &[],
            )
            .await
        }
    });
    let process = process.unwrap();
    wait_file(&root.join("root-ready"));
    fs::write(root.join("worker-ready"), "ready").unwrap();
    wait_file(&root.join("parent-observed"));
    match ending.as_str() {
        "handle" => process.session.request_terminate(),
        "cancel" => {
            cancel_desktop_commands().unwrap();
            cancel_desktop_commands().unwrap();
            supervisor.check_health().unwrap();
            let fresh = runtime
                .block_on(crate::spawn_pipe_process(
                    "/bin/true",
                    &[],
                    &root,
                    &HashMap::new(),
                    &None,
                    &[],
                ))
                .unwrap();
            assert_eq!(runtime.block_on(fresh.exit_rx).unwrap(), 0);
        }
        "broker-death" => {
            channel::signal(&supervisor.pidfd, libc::SIGKILL).unwrap();
            assert!(channel::poll(supervisor.pidfd.as_raw_fd(), 3_000).unwrap());
            assert!(supervisor.check_health().is_err());
            assert!(require_desktop_command_supervision().is_err());
        }
        "engine-death" => {
            std::mem::forget(process);
            loop {
                std::thread::sleep(Duration::from_secs(1));
            }
        }
        "drop" => {}
        _ => panic!("unknown fixture ending"),
    }
    // Omit the ordinary handle destructor: supervisor cleanup must stand alone.
    std::mem::forget(process);
    drop(supervisor);
}

#[test]
fn pipe_and_owned_pty_retire_exact_births_without_touching_other_children() {
    for backend in ["pipe", "pty"] {
        for ending in ["handle", "cancel", "drop", "broker-death", "engine-death"] {
            let root = tempfile::tempdir().unwrap();
            let mut control = Command::new("/bin/sleep").arg("60").spawn().unwrap();
            let control_birth = channel::pidfd(control.id() as libc::pid_t).unwrap();
            let mut worker = fixture("worker_fixture")
                .env("AIRS_PTY_FIXTURE_ROOT", root.path())
                .env("AIRS_PTY_FIXTURE_BACKEND", backend)
                .env("AIRS_PTY_FIXTURE_ENDING", ending)
                .stdout(Stdio::null())
                .spawn()
                .unwrap();
            wait_file(&root.path().join("worker-ready"));
            let pid = fs::read_to_string(root.path().join("root-ready"))
                .unwrap()
                .parse()
                .unwrap();
            let birth = channel::pidfd(pid).unwrap();
            fs::write(root.path().join("parent-observed"), "ready").unwrap();
            if ending == "engine-death" {
                worker.kill().unwrap();
                assert!(!worker.wait().unwrap().success());
            } else {
                assert!(worker.wait().unwrap().success(), "{backend}: {ending}");
            }
            assert!(
                channel::poll(birth.as_raw_fd(), 3_000).unwrap(),
                "{backend}: {ending}"
            );
            fs::write(root.path().join("release-after-stop"), "release").unwrap();
            assert!(!root.path().join("escaped-marker").exists());
            assert!(!channel::poll(control_birth.as_raw_fd(), 0).unwrap());
            control.kill().unwrap();
            control.wait().unwrap();
        }
    }
}

#[test]
fn interrupted_spawn_retires_supervision_before_another_command() {
    for backend in ["pipe", "pty"] {
        let root = tempfile::tempdir().unwrap();
        assert!(
            fixture("worker_fixture")
                .env("AIRS_PTY_FIXTURE_ROOT", root.path())
                .env("AIRS_PTY_FIXTURE_BACKEND", backend)
                .env("AIRS_PTY_FIXTURE_ENDING", "spawn-error")
                .stdout(Stdio::null())
                .status()
                .unwrap()
                .success()
        );
    }
}
