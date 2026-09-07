//! Public-client OIDC for AIRS; access JWTs retain their original issuer signature.
mod provider;
pub use provider::IdentityConfig;
pub use provider::Provider;
