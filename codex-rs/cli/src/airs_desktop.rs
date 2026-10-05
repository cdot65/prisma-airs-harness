//! Owned, bounded desktop bootstrap. This command never reads a saved identity.
use anyhow::Context;
use clap::Args as ClapArgs;
use codex_arg0::Arg0DispatchPaths;
use serde::Deserialize;
use serde::Serialize;
use std::path::PathBuf;
use tokio::io::AsyncBufRead;
use tokio::io::AsyncBufReadExt;
use zeroize::Zeroize;
use zeroize::Zeroizing;

#[path = "airs_desktop_runtime.rs"]
mod runtime;
#[path = "airs_desktop_transport.rs"]
mod transport;

pub(super) const FRAME_LIMIT: usize = 64 * 1024;
pub(super) const REPLAY_LIMIT: usize = 4 * 1024 * 1024;

#[derive(Debug, ClapArgs)]
pub(super) struct Args {
    /// Permit HTTP only for an offline loopback fixture.
    #[arg(long, hide = true)]
    pub allow_http_loopback: bool,
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) enum LocalAccess {
    #[default]
    ReadOnly,
    WorkspaceWrite,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ContextBinding {
    pub tenant_id: uuid::Uuid,
    pub workspace_id: uuid::Uuid,
    pub credential_id: uuid::Uuid,
    pub credential_epoch: u64,
    pub gateway_endpoint: String,
    pub project_path: Option<PathBuf>,
    #[serde(default)]
    pub local_access: LocalAccess,
    pub gateway_config: Option<String>,
    pub model: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Bootstrap {
    v: u32,
    tenant_id: uuid::Uuid,
    workspace_id: uuid::Uuid,
    credential_id: uuid::Uuid,
    credential_epoch: u64,
    gateway_endpoint: String,
    api_key: String,
    project_path: Option<PathBuf>,
    #[serde(default)]
    local_access: LocalAccess,
    gateway_config: Option<String>,
    model: Option<String>,
}

impl Drop for Bootstrap {
    fn drop(&mut self) {
        self.api_key.zeroize();
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BootstrapRequest {
    id: i64,
    method: String,
    params: Bootstrap,
}

pub(super) async fn read_frame<R: AsyncBufRead + Unpin>(
    reader: &mut R,
    limit: usize,
) -> anyhow::Result<Option<Zeroizing<Vec<u8>>>> {
    let mut bytes = Zeroizing::new(Vec::new());
    loop {
        let available = reader.fill_buf().await?;
        if available.is_empty() {
            anyhow::ensure!(bytes.is_empty(), "Incomplete desktop frame");
            return Ok(None);
        }
        let newline = available.iter().position(|byte| *byte == b'\n');
        let count = newline.map_or(available.len(), |index| index + 1);
        anyhow::ensure!(
            bytes.len() + count <= limit,
            "Desktop frame exceeded its limit"
        );
        bytes.extend_from_slice(&available[..count]);
        reader.consume(count);
        if newline.is_some() {
            return Ok(Some(bytes));
        }
    }
}

pub(super) async fn run(arg0_paths: Arg0DispatchPaths, args: &Args) -> anyhow::Result<()> {
    // Trace bundles contain plaintext conversations independently of the thread store.
    anyhow::ensure!(
        std::env::var_os("CODEX_ROLLOUT_TRACE_ROOT").is_none(),
        "Desktop trace recording must be disabled"
    );
    // Desktop owns a pipe, not an interactive terminal. A blocking stdin task
    // cannot be cancelled when supervision loss retires the runtime while the
    // native host still has its writer open.
    #[cfg(unix)]
    let stdin = {
        use std::os::fd::AsFd;
        tokio::net::unix::pipe::Receiver::from_owned_fd(
            std::io::stdin().as_fd().try_clone_to_owned()?,
        )?
    };
    #[cfg(not(unix))]
    let stdin = tokio::io::stdin();
    let mut input = tokio::io::BufReader::new(stdin);
    let bootstrap = read_frame(&mut input, FRAME_LIMIT).await;
    let result = match bootstrap {
        Ok(Some(bytes)) => match serde_json::from_slice::<BootstrapRequest>(&bytes) {
            Ok(request) => runtime::bootstrap(arg0_paths, args, request).await,
            Err(_) => Err(anyhow::anyhow!("Invalid desktop bootstrap")),
        },
        Ok(None) => return Ok(()),
        Err(_) => Err(anyhow::anyhow!("Invalid desktop bootstrap")),
    };
    match result {
        Ok((runtime, context, secret)) => transport::serve(input, runtime, context, secret).await,
        Err(_) => {
            transport::write_json(&mut tokio::io::stdout(), &serde_json::json!({"id":0,"error":{"code":-32602,"message":"Desktop bootstrap rejected"}}), "").await?;
            Err(anyhow::anyhow!("Desktop bootstrap rejected"))
        }
    }
}

impl BootstrapRequest {
    fn validate(&self) -> anyhow::Result<ContextBinding> {
        anyhow::ensure!(
            self.id == 0 && self.method == "desktop/bootstrap" && self.params.v == 1,
            "Unsupported desktop bootstrap"
        );
        let p = &self.params;
        anyhow::ensure!(
            p.gateway_endpoint.len() <= 2048
                && p.model.as_ref().is_none_or(|model| model.len() <= 256),
            "Desktop routing selection exceeded its bounds"
        );
        anyhow::ensure!(
            !p.api_key.is_empty()
                && p.api_key.len() <= 16 * 1024
                && !p.api_key.chars().any(char::is_control),
            "Invalid Gateway credential"
        );
        anyhow::ensure!(
            !p.tenant_id.is_nil() && !p.workspace_id.is_nil() && !p.credential_id.is_nil(),
            "Invalid desktop identity"
        );
        anyhow::ensure!(
            p.local_access == LocalAccess::ReadOnly || p.project_path.is_some(),
            "Local writes require a project"
        );
        let project_path = p
            .project_path
            .as_ref()
            .map(|path| {
                anyhow::ensure!(path.is_absolute(), "Project path must be absolute");
                let resolved = path.canonicalize().context("Project path is unavailable")?;
                anyhow::ensure!(resolved.is_dir(), "Project path must be a directory");
                Ok(resolved)
            })
            .transpose()?;
        if let Some(config) = &p.gateway_config {
            codex_model_provider_info::GatewayRouting::validate_saved_config(config)
                .map_err(anyhow::Error::msg)?;
        }
        Ok(ContextBinding {
            tenant_id: p.tenant_id,
            workspace_id: p.workspace_id,
            credential_id: p.credential_id,
            credential_epoch: p.credential_epoch,
            gateway_endpoint: p.gateway_endpoint.clone(),
            project_path,
            local_access: p.local_access,
            gateway_config: p.gateway_config.clone(),
            model: p.model.clone(),
        })
    }
}

#[cfg(test)]
#[path = "airs_desktop_tests.rs"]
mod tests;
