//! Recognize canonical owned MCP helpers without executing or interpreting a shell.
use super::Binding;
use super::Source;
use super::helper_command;
use crate::airs_helper_relocation;
use anyhow::Context;
use std::path::Path;
use std::path::PathBuf;
use uuid::Uuid;

pub(crate) struct Owned {
    pub(crate) id: Uuid,
}

fn saved_executable(command: &str, home: &Path) -> Option<(PathBuf, Uuid)> {
    #[cfg(not(windows))]
    let (quoted, id) = {
        let (prefix, id) = command.rsplit_once(" --binding ")?;
        let suffix = format!(
            " mcp-credential --home {}",
            super::shell_quote(home.to_str()?)
        );
        (
            prefix.strip_suffix(&suffix)?.to_owned(),
            Uuid::parse_str(id).ok()?,
        )
    };
    #[cfg(windows)]
    let (quoted, id) = {
        use base64::Engine;
        let encoded =
            command.strip_prefix("powershell.exe -NoProfile -NonInteractive -EncodedCommand ")?;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .ok()?;
        if !bytes.len().is_multiple_of(2) {
            return None;
        }
        let script = String::from_utf16(
            &bytes
                .chunks_exact(2)
                .map(|v| u16::from_le_bytes([v[0], v[1]]))
                .collect::<Vec<_>>(),
        )
        .ok()?;
        let script = script
            .strip_prefix("& ")?
            .strip_suffix("'; exit $LASTEXITCODE")?;
        let (prefix, id) = script.rsplit_once(" --binding '")?;
        let suffix = format!(
            " mcp-credential --home '{}'",
            home.to_str()?.replace('\'', "''")
        );
        (
            prefix.strip_suffix(&suffix)?.to_owned(),
            Uuid::parse_str(id).ok()?,
        )
    };
    let inner = quoted.strip_prefix('\'')?.strip_suffix('\'')?;
    #[cfg(not(windows))]
    let executable = PathBuf::from(inner.replace("'\\''", "'"));
    #[cfg(windows)]
    let executable = PathBuf::from(inner.replace("''", "'"));
    if !airs_helper_relocation::owned_executable(&executable)
        || helper_command(&executable, home, id).ok()?.as_str() != command
    {
        return None;
    }
    Some((executable, id))
}

pub(crate) fn owned(
    home: &Path,
    name: &str,
    server: &toml::Value,
) -> anyhow::Result<Option<Owned>> {
    let Some(helper) = server.get("http_headers_helper") else {
        return Ok(None);
    };
    let command = helper
        .as_str()
        .context("MCP HTTP headers helper must be a command string")?;
    if command.len() > 32_768 {
        return Ok(None);
    }
    let Some((_, id)) = saved_executable(command, home) else {
        return Ok(None);
    };
    let path = home.join("mcp-bindings").join(format!("{id}.json"));
    // A custom command with no owned binding must remain literal.
    if !path.try_exists()? {
        return Ok(None);
    }
    let binding: Binding = serde_json::from_slice(&airs_helper_relocation::read(&path, 16_384)?)
        .map_err(|_| anyhow::anyhow!("invalid managed MCP binding"))?;
    anyhow::ensure!(
        binding.schema_version == 1
            && binding.id == id
            && binding.server == name
            && server.get("url").and_then(toml::Value::as_str) == Some(binding.url.as_str())
            && (binding.oidc.is_some() != binding.credential_file.is_some()),
        "managed MCP binding differs from configuration"
    );
    if let Some(identity) = &binding.oidc {
        let inference = crate::airs_credentials::read_binding(home)?;
        let (Some(Source::Oidc { identity: user }), Some(Source::Oidc { identity: mcp })) =
            (&inference.source, &identity.source)
        else {
            anyhow::bail!("managed MCP identity is inconsistent");
        };
        anyhow::ensure!(
            identity.schema_version == 1
                && identity.id == id
                && identity.gateway_url == binding.url
                && identity.credential_fingerprint == binding.credential_fingerprint
                && user.subject == mcp.subject
                && user.config.issuer == mcp.config.issuer
                && user.config.client_id != mcp.config.client_id
                && user.config.audience != mcp.config.audience,
            "managed MCP identity differs from inference"
        );
    } else {
        anyhow::ensure!(
            binding
                .credential_file
                .as_deref()
                .context("missing MCP credential source")?
                .is_absolute(),
            "managed MCP credential file must be absolute"
        );
    }
    Ok(Some(Owned { id }))
}
