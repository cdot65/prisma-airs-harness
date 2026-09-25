use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

/// Separates a gateway's default route from the local model capability catalog.
///
/// `default_route` identifies a local capability descriptor, not a provider model.
/// Selecting it omits the inference model field. Other selections must name an
/// explicit gateway integration and model; authorization remains server-side.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GatewayRouting {
    pub default_route: String,
}

impl GatewayRouting {
    /// Accept only bounded saved configuration identifiers, never inline JSON.
    /// This validates syntax; the gateway retains all authorization decisions.
    pub fn validate_saved_config(selection: &str) -> Result<(), String> {
        if selection.is_empty()
            || selection.len() > 128
            || !selection
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        {
            return Err("select a saved gateway config ID (1–128 letters, digits, dots, underscores or hyphens)".into());
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.default_route.is_empty()
            || self.default_route.starts_with('@')
            || self
                .default_route
                .chars()
                .any(|c| c.is_whitespace() || c.is_control())
        {
            return Err(
                "gateway.default_route must be a nonempty local capability ID, not an @provider/model"
                    .to_string(),
            );
        }
        Ok(())
    }

    pub fn request_model(&self, selection: &str) -> Result<Option<String>, String> {
        self.validate()?;
        if selection == self.default_route {
            return Ok(None);
        }
        let qualified = selection.strip_prefix('@').and_then(|v| v.split_once('/'));
        if let Some((provider, model)) = qualified
            && !provider.is_empty()
            && !model.is_empty()
            && !selection
                .chars()
                .any(|c| c.is_whitespace() || c.is_control())
            && provider
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        {
            return Ok(Some(selection.to_string()));
        }
        Err("select the gateway default route or an explicit @provider/model".to_string())
    }
}

#[cfg(test)]
#[path = "gateway_tests.rs"]
mod tests;
