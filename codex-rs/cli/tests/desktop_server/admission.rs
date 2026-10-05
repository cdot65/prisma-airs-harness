//! Desktop-only single-process admission through actual model calls/approval.
use super::*;
use pretty_assertions::assert_eq;

#[derive(Clone, Copy)]
enum Terminal {
    Pipe,
    Pty,
}
#[derive(Clone, Copy)]
enum Probe {
    Direct,
    Fork,
    Vfork,
    Spawn,
    DoubleFork,
    Runner,
    Pipeline,
}

#[tokio::test]
async fn approved_external_exec_replacement_succeeds_in_pipe_and_pty() -> Result<()> {
    for terminal in [Terminal::Pipe, Terminal::Pty] {
        admission_probe(Probe::Direct, terminal).await?;
    }
    Ok(())
}

#[tokio::test]
async fn approved_fork_vfork_spawn_and_double_fork_are_denied_before_child_execution() -> Result<()>
{
    for probe in [Probe::Fork, Probe::Vfork, Probe::Spawn, Probe::DoubleFork] {
        for terminal in [Terminal::Pipe, Terminal::Pty] {
            admission_probe(probe, terminal).await?;
        }
    }
    Ok(())
}

#[tokio::test]
async fn approved_pipeline_and_build_runner_cannot_create_children_or_sideeffects() -> Result<()> {
    for probe in [Probe::Pipeline, Probe::Runner] {
        for terminal in [Terminal::Pipe, Terminal::Pty] {
            admission_probe(probe, terminal).await?;
        }
    }
    Ok(())
}

async fn admission_probe(probe: Probe, terminal: Terminal) -> Result<()> {
    let temp = tempfile::tempdir_in("/private/tmp")?;
    let project = temp.path().join("project");
    std::fs::create_dir(&project)?;
    compile_admission_fixture(&project)?;
    let (command, expected) = match probe {
        Probe::Direct => ("exec ./admission-fixture direct", Some("DIRECT_EXEC_OK")),
        Probe::Fork => ("exec ./admission-fixture fork", Some("FORK_DENIED_EPERM")),
        Probe::Vfork => ("exec ./admission-fixture vfork", Some("VFORK_DENIED_EPERM")),
        Probe::Spawn => ("exec ./admission-fixture spawn", Some("SPAWN_DENIED_EPERM")),
        Probe::DoubleFork => (
            "exec ./admission-fixture double-fork",
            Some("DOUBLE_FORK_DENIED_EPERM"),
        ),
        Probe::Runner => (
            "exec ./admission-fixture runner",
            Some("BUILD_RUNNER_UNSUPPORTED_EPERM"),
        ),
        Probe::Pipeline => (
            "exec /bin/sh -c './admission-fixture child pipeline-escaped | /bin/cat'",
            None,
        ),
    };
    let server = MockServer::start().await;
    let tool = sse(vec![
        json!({"type":"response.created","response":{"id":"admission-response"}}),
        json!({"type":"response.output_item.done","item":{"type":"function_call","id":"admission-item","call_id":"admission-call","name":"exec_command","arguments":json!({"cmd":command,"login":false,"tty":matches!(terminal, Terminal::Pty),"yield_time_ms":1000,"max_output_tokens":1000}).to_string()}}),
        json!({"type":"response.completed","response":{"id":"admission-response","usage":{"input_tokens":1,"output_tokens":1,"total_tokens":2}}}),
    ])?;
    Mock::given(method("POST"))
        .and(path("/v1/responses"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(tool, "text/event-stream"))
        .with_priority(1)
        .up_to_n_times(1)
        .mount(&server)
        .await;
    // The second request can only occur after the real tool result, so the
    // ordinary model continuation is a barrier rather than a timing guess.
    Mock::given(method("POST"))
        .and(path("/v1/responses"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(completed()?, "text/event-stream"))
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
    let thread = desktop.thread().await?;
    desktop.send(json!({"id":20,"method":"turn/start","params":{"threadId":thread,"input":[{"type":"text","text":"Run the approved single-process admission fixture","text_elements":[]}]}})).await?;
    anyhow::ensure!(
        desktop.response(20).await?.get("result").is_some(),
        "admission turn rejected"
    );
    let mut approvals = 0;
    loop {
        let event = desktop.next().await?;
        if event["method"] == "item/commandExecution/requestApproval" {
            approvals += 1;
            desktop
                .send(json!({"id":event["id"],"result":{"decision":"accept"}}))
                .await?;
        }
        if event["method"] == "turn/completed" {
            break;
        }
    }
    assert_eq!(
        approvals, 1,
        "single command requires one actual native approval"
    );
    let requests = server
        .received_requests()
        .await
        .context("admission requests")?;
    assert_eq!(
        requests.len(),
        2,
        "denial must not retry a command or inference"
    );
    for request in &requests {
        anyhow::ensure!(
            !String::from_utf8_lossy(&request.body).contains(KEY),
            "credential reached model context"
        );
    }
    let first = String::from_utf8_lossy(&requests[0].body);
    anyhow::ensure!(first.contains("child process creation, pipelines, and build/test runners that spawn workers are unavailable"), "static unsupported scope hint missing");
    anyhow::ensure!(
        first.contains("never retry outside the sandbox or escalate authority"),
        "static no-retry instruction missing"
    );
    let response: Value = requests[1].body_json()?;
    let output = response["input"]
        .as_array()
        .context("admission input")?
        .iter()
        .find(|item| item["type"] == "function_call_output")
        .context("admission tool result")?["output"]
        .as_str()
        .context("admission output")?;
    anyhow::ensure!(
        output.len() <= 8192,
        "admission result exceeded its bounded fixture output"
    );
    // A denied spawn must produce no child-side work even while the engine
    // remains alive. Release any accidentally admitted delayed worker first.
    std::fs::write(
        project.join("release-delayed-child"),
        "fixture continuation reached",
    )?;
    tokio::time::sleep(Duration::from_millis(250)).await;
    for marker in [
        "forbidden-child-ready",
        "fork-escaped",
        "vfork-escaped",
        "spawn-escaped",
        "double-fork-escaped",
        "runner-escaped",
        "pipeline-escaped",
    ] {
        anyhow::ensure!(
            !project.join(marker).exists(),
            "denied child produced a forbidden project sideeffect"
        );
    }
    if let Some(expected) = expected {
        anyhow::ensure!(
            project.join(expected).exists() && output.contains(expected),
            "direct execution or literal EPERM admission proof missing"
        );
        anyhow::ensure!(
            output.contains("Process exited with code 0"),
            "bounded admission fixture did not finish successfully"
        );
    } else {
        // This tool's normal result carries the process code in its bounded
        // output, rather than a native commandExecution item projection.
        let code = output
            .lines()
            .find_map(|line| line.strip_prefix("Process exited with code "))
            .and_then(|code| code.trim().parse::<i32>().ok())
            .with_context(|| {
                format!(
                    "pipeline process exit status unavailable: bounded fixture output={:?}",
                    output.chars().take(1024).collect::<String>()
                )
            })?;
        anyhow::ensure!(code != 0, "child-spawning pipeline unexpectedly succeeded");
        anyhow::ensure!(
            output.contains("fork: Operation not permitted"),
            "pipeline lacked actual fork denial evidence"
        );
    }
    desktop.stop().await?;
    Ok(())
}

fn compile_admission_fixture(project: &Path) -> Result<()> {
    const SOURCE: &str = r#"
#include <errno.h>
#include <fcntl.h>
#include <spawn.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>
extern char **environ;
static void marker(const char *name) {
    int file = open(name, O_CREAT | O_EXCL | O_WRONLY, 0600);
    if (file < 0) _exit(70);
    close(file);
}
static void delayed_child(const char *name) {
    marker("forbidden-child-ready");
    for (int n = 0; n < 1000 && access("release-delayed-child", F_OK) != 0; n++) usleep(10000);
    usleep(50000);
    marker(name);
    _exit(0);
}
static int denied(int error, const char *flag) {
    if (error != EPERM) { puts("UNEXPECTED_ADMISSION_RESULT"); return 76; }
    marker(flag);
    puts(flag);
    return 0;
}
int main(int argc, char **argv) {
    if (argc < 2) return 71;
    if (strcmp(argv[1], "child") == 0 && argc == 3) delayed_child(argv[2]);
    if (strcmp(argv[1], "direct") == 0) {
        marker("DIRECT_EXEC_OK"); puts("DIRECT_EXEC_OK"); return 0;
    }
    if (strcmp(argv[1], "spawn") == 0 || strcmp(argv[1], "runner") == 0) {
        int runner = strcmp(argv[1], "runner") == 0;
        char *args[] = {"./admission-fixture", "child", runner ? "runner-escaped" : "spawn-escaped", NULL};
        pid_t child;
        int error = posix_spawn(&child, args[0], NULL, NULL, args, environ);
        return denied(error, runner ? "BUILD_RUNNER_UNSUPPORTED_EPERM" : "SPAWN_DENIED_EPERM");
    }
    if (strcmp(argv[1], "vfork") == 0) {
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wdeprecated-declarations"
        /* Exercise the legacy API itself; its deprecation is intentional. */
        pid_t child = vfork();
#pragma clang diagnostic pop
        if (child == 0) {
            execl("./admission-fixture", "admission-fixture", "child", "vfork-escaped", (char *)NULL);
            _exit(77);
        }
        return denied(child < 0 ? errno : 0, "VFORK_DENIED_EPERM");
    }
    int twice = strcmp(argv[1], "double-fork") == 0;
    pid_t child = fork();
    if (child == 0) {
        if (twice) {
            marker("forbidden-child-ready");
            if (setsid() < 0) _exit(78);
            pid_t grandchild = fork();
            if (grandchild < 0) _exit(79);
            if (grandchild > 0) _exit(0);
            while (access("release-delayed-child", F_OK) != 0) usleep(10000);
            usleep(50000);
            marker("double-fork-escaped");
            _exit(0);
        }
        delayed_child("fork-escaped");
    }
    return denied(child < 0 ? errno : 0, twice ? "DOUBLE_FORK_DENIED_EPERM" : "FORK_DENIED_EPERM");
}
"#;
    let source = project.join("admission-fixture.c");
    std::fs::write(&source, SOURCE)?;
    let status = std::process::Command::new("/usr/bin/cc")
        .args(["-Wall", "-Wextra", "-Werror"])
        .arg(source)
        .arg("-o")
        .arg(project.join("admission-fixture"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    anyhow::ensure!(
        status.success(),
        "local admission fixture compilation failed"
    );
    Ok(())
}
