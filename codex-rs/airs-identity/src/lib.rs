//! Public-client OIDC for AIRS; access JWTs retain their original issuer signature.
mod provider;
pub use provider::IdentityConfig;
pub use provider::Provider;
mod tokens;
pub use tokens::Identity;
pub use tokens::Tokens;
mod browser;
mod device;
pub use browser::BrowserLogin;
pub use device::DeviceLogin;
