//! Bounded, cancellable private interaction with the exact running AIRS executable.
use super::Event;
use super::Operation;
use crate::app_event::AppEvent;
use crate::app_event_sender::AppEventSender;
use std::path::Path;
use std::process::Stdio;
use tokio::io::AsyncBufRead;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::sync::oneshot;

pub(crate) fn validate_new_connection(name: &str, endpoint: &str) -> Result<(), String> {
    if name.is_empty()
        || name.len() > 64
        || !name
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_'))
    {
        return Err(
            "Use 1–64 letters, digits, underscores or hyphens for the connection name.".into(),
        );
    }
    let url = url::Url::parse(endpoint)
        .map_err(|_| "Enter the full HTTPS AI Gateway MCP URL.".to_string())?;
    if endpoint.len() > 2048
        || !endpoint.bytes().all(|c| c.is_ascii_graphic())
        || url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(
            "Use an HTTPS AI Gateway MCP URL without credentials, query parameters or fragments."
                .into(),
        );
    }
    Ok(())
}

pub(crate) fn arguments(operation: &Operation) -> Vec<String> {
    let mut args = vec!["mcp".into()];
    match operation {
        Operation::Add { name, url } => args.extend([
            "add".into(),
            "--no-browser".into(),
            "--scopes".into(),
            "mcp:servers:read,mcp:tools:list,mcp:tools:call".into(),
            "--url".into(),
            url.clone(),
            "--".into(),
            name.clone(),
        ]),
        Operation::Login(name) => args.extend([
            "login".into(),
            "--no-browser".into(),
            "--".into(),
            name.clone(),
        ]),
        Operation::Logout(name) => args.extend(["logout".into(), "--".into(), name.clone()]),
        Operation::Remove(name) => args.extend(["remove".into(), "--".into(), name.clone()]),
        Operation::Verify(_) => {}
    }
    args
}

// Keep partial input outside this future: select! cancellation must not lose bytes.
async fn bounded_line(
    reader: &mut (impl AsyncBufRead + Unpin),
    line: &mut Vec<u8>,
) -> Result<Option<String>, String> {
    loop {
        let bytes = reader
            .fill_buf()
            .await
            .map_err(|_| "MCP interaction closed unexpectedly.")?;
        if bytes.is_empty() && line.is_empty() {
            return Ok(None);
        }
        let end = bytes.iter().position(|b| *b == b'\n').map(|i| i + 1);
        let count = end.unwrap_or(bytes.len());
        if line.len() + count > 32 * 1024 {
            return Err("MCP interaction exceeded its size limit.".into());
        }
        line.extend_from_slice(&bytes[..count]);
        let finished = end.is_some() || bytes.is_empty();
        reader.consume(count);
        if finished {
            return String::from_utf8(std::mem::take(line))
                .map(Some)
                .map_err(|_| "Invalid MCP interaction encoding.".into());
        }
    }
}

pub(crate) fn authorization_url(line: &str) -> Result<Option<String>, String> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
        return Ok(None);
    };
    if value.get("airs_mcp").and_then(serde_json::Value::as_u64) != Some(1) {
        return Ok(None);
    }
    let value = value
        .get("authorization_url")
        .and_then(serde_json::Value::as_str)
        .ok_or("Invalid MCP authorization response.")?;
    let url = url::Url::parse(value).map_err(|_| "Invalid MCP authorization URL.")?;
    if value.len() > 16384
        || value.chars().any(char::is_control)
        || url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return Err("The gateway returned an unsafe authorization URL.".into());
    }
    Ok(Some(value.to_string()))
}

pub(crate) async fn run(
    home: &Path,
    cwd: &Path,
    operation: &Operation,
    attempt: u64,
    thread: codex_protocol::ThreadId,
    tx: &AppEventSender,
) -> Result<(), String> {
    if matches!(operation, Operation::Verify(_)) {
        return Ok(());
    }
    #[cfg(target_os = "linux")]
    let executable = std::path::PathBuf::from("/proc/self/exe");
    #[cfg(not(target_os = "linux"))]
    let executable = std::env::current_exe().map_err(|_| "Cannot locate the running harness.")?;
    let mut child = tokio::process::Command::new(executable)
        .args(arguments(operation))
        .current_dir(cwd)
        .env("AIRS_HARNESS_HOME", home)
        .env("AIRS_MCP_INTERACTION", "json-v1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .map_err(|_| "Could not start the MCP operation.")?;
    let mut stdin = child.stdin.take().ok_or("MCP input unavailable.")?;
    let mut reader = BufReader::new(child.stdout.take().ok_or("MCP output unavailable.")?);
    let mut callback: Option<oneshot::Receiver<String>> = None;
    let mut buffer = Vec::new();
    let mut total = 0;
    loop {
        tokio::select! {
            line = bounded_line(&mut reader, &mut buffer) => {
                let Some(line) = line? else { break; };
                total += line.len();
                if total > 256 * 1024 { return Err("MCP interaction exceeded its size limit.".into()); }
                if let Some(url) = authorization_url(&line)? {
                    let (sender, receiver) = oneshot::channel();
                    callback = Some(receiver);
                    tx.send(AppEvent::AirsMcpManager(Event::Authorize { attempt, thread, url, callback: sender }));
                }
            }
            result = async {
                match callback.as_mut() {
                    Some(receiver) => receiver.await,
                    None => std::future::pending().await,
                }
            } => {
                let value = result.map_err(|_| "MCP sign-in cancelled. The saved connection is preserved.")?;
                if value.len() > 65536 || value.contains(['\r', '\n']) { return Err("Invalid callback input.".into()); }
                stdin.write_all(value.as_bytes()).await.map_err(|_| "MCP sign-in is no longer waiting for input.")?;
                stdin.write_all(b"\n").await.map_err(|_| "MCP sign-in is no longer waiting for input.")?;
                callback = None;
            }
        }
    }
    if child
        .wait()
        .await
        .map_err(|_| "MCP operation did not complete.")?
        .success()
    {
        Ok(())
    } else {
        Err("MCP operation did not complete. Check the gateway URL, permission and unlocked credential store, then retry from /mcp. A saved connection may still need sign-in.".into())
    }
}

#[cfg(test)]
#[path = "process_tests.rs"]
mod tests;
