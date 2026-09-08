//! Public-client OIDC for AIRS. The original access JWT is the gateway credential.
//! Bearer material intentionally implements neither Debug nor Display.
mod browser;
mod device;
mod provider;
mod tokens;

pub use browser::BrowserLogin;
pub use device::DeviceLogin;
pub use provider::IdentityConfig;
pub use provider::Provider;
pub use tokens::Identity;
pub use tokens::Tokens;

mod storage;
pub use storage::CredentialStore;
pub use storage::WorkspaceCredentialFormat;
pub use storage::WorkspaceCredentialStore;
