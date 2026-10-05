//! Actual approved command lifetime at the owned desktop process boundary.
use super::*;
use pretty_assertions::assert_eq;

#[derive(Clone, Copy)]
enum Stop {
    Crash,
    SupervisorCrash,
    Eof,
    Interrupt,
}

#[tokio::test]
async fn abrupt_engine_death_stops_approved_single_process_pipe_and_pty_commands() -> Result<()> {
    shutdown_probe(Stop::Crash).await
}

#[tokio::test]
async fn abrupt_supervisor_death_stops_approved_single_process_pipe_and_pty_commands() -> Result<()>
{
    shutdown_probe(Stop::SupervisorCrash).await
}

#[tokio::test]
async fn stdin_eof_stops_approved_single_process_pipe_and_pty_commands() -> Result<()> {
    shutdown_probe(Stop::Eof).await
}

#[tokio::test]
async fn native_interrupt_stops_approved_single_process_pipe_and_pty_commands() -> Result<()> {
    shutdown_probe(Stop::Interrupt).await
}

#[tokio::test]
async fn native_interrupt_acknowledges_a_completed_turn_after_local_cleanup() -> Result<()> {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/responses"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(completed()?, "text/event-stream"))
        .mount(&server)
        .await;
    let temp = tempfile::tempdir()?;
    let mut desktop = Desktop::start(temp.path(), &format!("{}/v1", server.uri()), None).await?;
    let thread = desktop.thread().await?;
    desktop.send(json!({"id":20,"method":"turn/start","params":{"threadId":thread,"input":[{"type":"text","text":"Complete the terminal-race fixture","text_elements":[]}]}})).await?;
    let turn = desktop.response(20).await?["result"]["turn"]["id"]
        .as_str()
        .context("terminal race turn")?
        .to_owned();
    let mut completed = desktop
        .seen
        .iter()
        .any(|event| event["method"] == "turn/completed");
    while !completed {
        completed = desktop.next().await?["method"] == "turn/completed";
    }
    desktop
        .send(json!({"id":30,"method":"turn/interrupt","params":{"threadId":thread,"turnId":turn}}))
        .await?;
    assert_eq!(desktop.response(30).await?, json!({"id":30,"result":{}}));
    desktop.send(json!({"id":31,"method":"turn/interrupt","params":{"threadId":"unowned-thread","turnId":turn}})).await?;
    anyhow::ensure!(
        desktop.response(31).await?.get("error").is_some(),
        "unowned interruption accepted"
    );
    desktop.stop().await?;
    Ok(())
}

#[tokio::test]
async fn supervision_loss_retires_runtime_while_native_stdout_is_not_drained() -> Result<()> {
    use std::os::fd::AsRawFd;

    let temp = tempfile::tempdir_in("/private/tmp")?;
    let project = temp.path().join("project");
    std::fs::create_dir(&project)?;
    compile_fixture(
        &project,
        r#"
#include <fcntl.h>
#include <string.h>
#include <unistd.h>
int main(void) {
    int file = open("command-started", O_CREAT | O_EXCL | O_WRONLY, 0600);
    if (file < 0) return 70;
    close(file);
    char output[8192];
    memset(output, 'x', sizeof(output));
    for (int n = 0; n < 1024; n++) {
        if (write(1, output, sizeof(output)) < 0) return 71;
    }
    sleep(20);
    return 0;
}
"#,
    )?;
    let server = MockServer::start().await;
    let command = sse(vec![
        json!({"type":"response.created","response":{"id":"pressure-response"}}),
        json!({"type":"response.output_item.done","item":{"type":"function_call","id":"pressure-item","call_id":"pressure-call","name":"exec_command","arguments":"{\"cmd\":\"exec ./lifetime-fixture\",\"login\":false,\"yield_time_ms\":1000}"}}),
        json!({"type":"response.completed","response":{"id":"pressure-response","usage":{"input_tokens":1,"output_tokens":1,"total_tokens":2}}}),
    ])?;
    Mock::given(method("POST"))
        .and(path("/v1/responses"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(command, "text/event-stream"))
        .mount(&server)
        .await;
    let mut desktop = Desktop::start_access(
        temp.path(),
        &format!("{}/v1", server.uri()),
        Some(&project),
        "workspaceWrite",
    )
    .await?;
    let supervisor = supervisor_pid(desktop.child.id().context("desktop PID")?)?;
    let thread = desktop.thread().await?;
    desktop.send(json!({"id":20,"method":"turn/start","params":{"threadId":thread,"input":[{"type":"text","text":"Run the approved output pressure fixture","text_elements":[]}]}})).await?;
    desktop.response(20).await?;
    loop {
        let event = desktop.next().await?;
        if event["method"] == "item/commandExecution/requestApproval" {
            desktop
                .send(json!({"id":event["id"],"result":{"decision":"accept"}}))
                .await?;
            break;
        }
    }
    // Leave both host pipe endpoints open while deliberately ceasing reads.
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let mut queued = 0;
            let ready = unsafe {
                libc::ioctl(
                    desktop.output.get_ref().as_raw_fd(),
                    libc::FIONREAD,
                    &raw mut queued,
                )
            };
            if project.join("command-started").exists() && ready == 0 && queued >= 8192 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .context("native stdout pressure did not develop")?;
    anyhow::ensure!(
        unsafe { libc::kill(supervisor, libc::SIGKILL) } == 0,
        "supervisor kill failed"
    );
    let status = tokio::time::timeout(Duration::from_secs(10), desktop.child.wait()).await??;
    anyhow::ensure!(
        !status.success(),
        "supervision loss did not retire under output pressure"
    );
    Ok(())
}

#[tokio::test]
async fn approved_sandboxed_command_cannot_signal_supervisor_or_unrelated_process() -> Result<()> {
    let temp = tempfile::tempdir_in("/private/tmp")?;
    let project = temp.path().join("project");
    std::fs::create_dir(&project)?;
    let server = MockServer::start().await;
    let mut desktop = Desktop::start_access(
        temp.path(),
        &format!("{}/v1", server.uri()),
        Some(&project),
        "workspaceWrite",
    )
    .await?;
    let supervisor = supervisor_pid(desktop.child.id().context("desktop PID")?)?;
    let mut unrelated = tokio::process::Command::new("/bin/sleep")
        .arg("30")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()?;
    let unrelated_pid = unrelated.id().context("unrelated fixture PID")?;
    // Shell builtins alone prove signal authority independently of fork policy.
    let script = format!(
        "kill -TERM {supervisor}; printf '%s\\n' \"$?\" > helper-signal-result; kill -TERM {unrelated_pid}; printf '%s\\n' \"$?\" > unrelated-signal-result"
    );
    let command = sse(vec![
        json!({"type":"response.created","response":{"id":"signal-response"}}),
        json!({"type":"response.output_item.done","item":{"type":"function_call","id":"signal-item","call_id":"signal-call","name":"exec_command","arguments":json!({"cmd":script,"login":false,"yield_time_ms":1000}).to_string()}}),
        json!({"type":"response.completed","response":{"id":"signal-response","usage":{"input_tokens":1,"output_tokens":1,"total_tokens":2}}}),
    ])?;
    Mock::given(method("POST"))
        .and(path("/v1/responses"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(command, "text/event-stream"))
        .with_priority(1)
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/responses"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(completed()?, "text/event-stream"))
        .with_priority(2)
        .mount(&server)
        .await;
    let thread = desktop.thread().await?;
    desktop.send(json!({"id":20,"method":"turn/start","params":{"threadId":thread,"input":[{"type":"text","text":"Run the approved signal boundary fixture","text_elements":[]}]}})).await?;
    desktop.response(20).await?;
    loop {
        let event = desktop.next().await?;
        if event["method"] == "item/commandExecution/requestApproval" {
            desktop
                .send(json!({"id":event["id"],"result":{"decision":"accept"}}))
                .await?;
        }
        if event["method"] == "turn/completed" {
            break;
        }
    }
    for marker in ["helper-signal-result", "unrelated-signal-result"] {
        anyhow::ensure!(
            std::fs::read_to_string(project.join(marker))?.trim() != "0",
            "approved sandboxed command signalled an unrelated process"
        );
    }
    anyhow::ensure!(
        unsafe { libc::kill(supervisor, 0) } == 0 && unrelated.try_wait()?.is_none(),
        "signal target did not remain alive"
    );
    unrelated.kill().await?;
    desktop.stop().await?;
    Ok(())
}

async fn shutdown_probe(stop: Stop) -> Result<()> {
    for tty in [false, true] {
        let temp = tempfile::tempdir_in("/private/tmp")?;
        let project = temp.path().join("project");
        std::fs::create_dir(&project)?;
        compile_lifetime_fixture(&project)?;
        let script = "exec ./lifetime-fixture";
        let command = sse(vec![
            json!({"type":"response.created","response":{"id":"shutdown-response"}}),
            json!({"type":"response.output_item.done","item":{"type":"function_call","id":"shutdown-item","call_id":"shutdown-call","name":"exec_command","arguments":json!({"cmd":script,"login":false,"tty":tty,"yield_time_ms":1000}).to_string()}}),
            json!({"type":"response.completed","response":{"id":"shutdown-response","usage":{"input_tokens":1,"output_tokens":1,"total_tokens":2}}}),
        ])?;
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/responses"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(command, "text/event-stream"))
            .with_priority(1)
            .up_to_n_times(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/v1/responses"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw(completed()?, "text/event-stream")
                    .set_delay(Duration::from_secs(30)),
            )
            .with_priority(2)
            .mount(&server)
            .await;
        let mut desktop = Desktop::start_access(
            temp.path(),
            &format!("{}/v1", server.uri()),
            Some(&project),
            "workspaceWrite",
        )
        .await?;
        let supervisor = supervisor_pid(desktop.child.id().context("desktop PID")?)?;
        let thread = desktop.thread().await?;
        desktop.send(json!({"id":20,"method":"turn/start","params":{"threadId":thread,"input":[{"type":"text","text":"Run the approved project lifetime fixture","text_elements":[]}]}})).await?;
        let turn = desktop.response(20).await?["result"]["turn"]["id"]
            .as_str()
            .context("shutdown turn")?
            .to_owned();
        loop {
            let event = desktop.next().await?;
            if event["method"] == "item/commandExecution/requestApproval" {
                desktop
                    .send(json!({"id":event["id"],"result":{"decision":"accept"}}))
                    .await?;
                break;
            }
        }
        tokio::time::timeout(Duration::from_secs(10), async {
            while !project.join("command-started").exists() {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .context("approved command did not start")?;
        // Establish admitted execution before testing each retirement boundary.
        tokio::time::sleep(Duration::from_millis(100)).await;
        let remaining = match stop {
            Stop::Crash => {
                let pid = desktop.child.id().context("desktop PID")?;
                // Match native process-group retirement, including SIGKILL
                // where no engine Drop/shutdown code can run.
                anyhow::ensure!(
                    unsafe { libc::kill(-(pid as i32), libc::SIGKILL) } == 0,
                    "owned engine group kill failed"
                );
                let status = desktop.child.wait().await?;
                anyhow::ensure!(!status.success(), "crash fixture unexpectedly succeeded");
                None
            }
            Stop::Eof => {
                desktop.stop().await?;
                None
            }
            Stop::SupervisorCrash => {
                anyhow::ensure!(
                    unsafe { libc::kill(supervisor, libc::SIGKILL) } == 0,
                    "owned supervisor kill failed"
                );
                let status =
                    tokio::time::timeout(Duration::from_secs(10), desktop.child.wait()).await??;
                anyhow::ensure!(
                    !status.success(),
                    "supervision loss did not retire the desktop runtime"
                );
                None
            }
            Stop::Interrupt => {
                desktop.send(json!({"id":40,"method":"turn/interrupt","params":{"threadId":thread,"turnId":turn}})).await?;
                anyhow::ensure!(
                    desktop.response(40).await?.get("result").is_some(),
                    "native interrupt rejected"
                );
                Some(desktop)
            }
        };
        // The delayed-write clock starts only after native stop completes.
        // Broad-suite scheduling cannot turn an earlier write into a false
        // post-stop assertion or finish the turn before admission is observed.
        std::fs::write(project.join("stop-now"), "native stop completed")?;
        tokio::time::sleep(Duration::from_millis(3500)).await;
        anyhow::ensure!(
            !project.join("parent-after-stop").exists(),
            "approved single-process command wrote after native stop (tty={tty})"
        );
        if let Some(desktop) = remaining {
            desktop.stop().await?;
        }
    }
    Ok(())
}

fn compile_lifetime_fixture(project: &Path) -> Result<()> {
    const SOURCE: &str = r#"
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
static void marker(const char *name) {
    int file = open(name, O_CREAT | O_EXCL | O_WRONLY, 0600);
    if (file < 0) _exit(70);
    dprintf(file, "%ld\n", (long)getpid());
    close(file);
}
int main(void) {
    marker("command-started");
    while (access("stop-now", F_OK) != 0) usleep(10000);
    usleep(3000000);
    marker("parent-after-stop");
    sleep(20);
    return 0;
}
"#;
    compile_fixture(project, SOURCE)
}

fn compile_fixture(project: &Path, source_code: &str) -> Result<()> {
    let source = project.join("lifetime-fixture.c");
    std::fs::write(&source, source_code)?;
    let status = std::process::Command::new("/usr/bin/cc")
        .arg(&source)
        .arg("-o")
        .arg(project.join("lifetime-fixture"))
        .status()?;
    anyhow::ensure!(
        status.success(),
        "local lifetime fixture compilation failed"
    );
    Ok(())
}

fn supervisor_pid(engine: u32) -> Result<libc::pid_t> {
    // Only enumerate this fixture engine's direct children. Never inspect
    // arbitrary process arguments or environment to identify the helper.
    let mut children = [0_i32; 16];
    let bytes = unsafe {
        libc::proc_listpids(
            6, // PROC_PPID_ONLY, omitted by libc's Darwin bindings.
            engine,
            children.as_mut_ptr().cast(),
            std::mem::size_of_val(&children) as libc::c_int,
        )
    };
    anyhow::ensure!(bytes > 0, "owned supervisor missing");
    let count = bytes as usize / std::mem::size_of::<libc::pid_t>();
    anyhow::ensure!(count < children.len(), "unexpected engine child count");
    let children = children[..count]
        .iter()
        .copied()
        .filter(|pid| *pid > 1)
        .collect::<Vec<_>>();
    anyhow::ensure!(children.len() == 1, "ambiguous owned supervisor");
    Ok(children[0])
}
