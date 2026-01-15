//! Auth client library for internal service authentication.
//!
//! Implements Bearer token validation with hardcoded credentials.
//! Based on EDR-0006: Internal Service Security.
//!
//! # Security Notice
//!
//! **Development/POC only.** Service tokens are hardcoded as compile-time constants.
//! This is intentional for the initial phase per [EDR-0006].
//!
//! Before production deployment:
//! 1. Move tokens to environment variables or a secrets manager
//! 2. Implement token rotation procedures
//! 3. Consider upgrading to centralized auth service (EDR-0006 Options 1 or 2)
//!
//! [EDR-0006]: ../../../docs/technical/decisions/EDR-0006-internal-service-security.md

pub mod model;

use std::collections::HashMap;
use std::sync::Arc;

use thiserror::Error;

pub use model::{ClientId, Permission, Role};

/// Authenticated service identity.
#[derive(Debug, Clone)]
pub struct ServiceIdentity {
    pub client_id: ClientId,
    pub permissions: &'static [Permission],
    pub roles: &'static [Role],
}

/// Credential entry for hardcoded authentication.
struct Credential {
    token: &'static str,
    client_id: ClientId,
    permissions: &'static [Permission],
    roles: &'static [Role],
}

/// Hardcoded service credentials.
const CREDENTIALS: &[Credential] = &[
    Credential {
        token: model::tokens::VENDOR_SOURCING,
        client_id: ClientId::VendorSourcing,
        permissions: &[Permission::FetchUrl],
        roles: &[],
    },
    Credential {
        token: model::tokens::RETRIEVER,
        client_id: ClientId::Retriever,
        permissions: &[],
        roles: &[],
    },
    Credential {
        token: model::tokens::CRAWLER,
        client_id: ClientId::Crawler,
        permissions: &[Permission::TestPermission],
        roles: &[Role::TestRole],
    },
];

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("missing authorization header")]
    MissingHeader,

    #[error("invalid authorization header format")]
    InvalidFormat,

    #[error("invalid token")]
    InvalidToken,
}

/// Auth client for validating Bearer tokens.
///
/// Uses `Arc` internally for efficient cloning - clones share the underlying
/// credentials map rather than duplicating it.
#[derive(Debug, Clone)]
pub struct AuthClient {
    /// Map of token -> ServiceIdentity (shared via Arc for efficient cloning)
    tokens: Arc<HashMap<String, ServiceIdentity>>,
}

impl AuthClient {
    /// Create a new auth client with hardcoded credentials.
    pub fn new() -> Self {
        let tokens = CREDENTIALS
            .iter()
            .map(|c| {
                (
                    c.token.to_string(),
                    ServiceIdentity {
                        client_id: c.client_id,
                        permissions: c.permissions,
                        roles: c.roles,
                    },
                )
            })
            .collect();

        Self {
            tokens: Arc::new(tokens),
        }
    }

    /// Validate a Bearer token from an Authorization header value.
    ///
    /// Expects format: "Bearer <token>"
    ///
    /// Returns the ServiceIdentity if valid.
    pub fn validate(&self, auth_header: &str) -> Result<&ServiceIdentity, AuthError> {
        let token = Self::extract_bearer_token(auth_header)?;
        self.validate_token(token)
    }

    /// Validate a token directly (without Bearer prefix).
    ///
    /// Returns the ServiceIdentity if valid.
    pub fn validate_token(&self, token: &str) -> Result<&ServiceIdentity, AuthError> {
        self.tokens.get(token).ok_or(AuthError::InvalidToken)
    }

    /// Extract the token from a "Bearer <token>" header value.
    fn extract_bearer_token(auth_header: &str) -> Result<&str, AuthError> {
        let parts: Vec<&str> = auth_header.splitn(2, ' ').collect();

        if parts.len() != 2 {
            return Err(AuthError::InvalidFormat);
        }

        if !parts[0].eq_ignore_ascii_case("bearer") {
            return Err(AuthError::InvalidFormat);
        }

        let token = parts[1].trim();
        if token.is_empty() {
            return Err(AuthError::InvalidFormat);
        }

        Ok(token)
    }
}

impl Default for AuthClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_vendor_sourcing_token() {
        let client = AuthClient::new();
        let header = format!("Bearer {}", model::tokens::VENDOR_SOURCING);
        let identity = client.validate(&header).unwrap();
        assert_eq!(identity.client_id, ClientId::VendorSourcing);
        assert!(identity.permissions.contains(&Permission::FetchUrl));
    }

    #[test]
    fn validate_retriever_token() {
        let client = AuthClient::new();
        let header = format!("Bearer {}", model::tokens::RETRIEVER);
        let identity = client.validate(&header).unwrap();
        assert_eq!(identity.client_id, ClientId::Retriever);
    }

    #[test]
    fn validate_crawler_token() {
        let client = AuthClient::new();
        let header = format!("Bearer {}", model::tokens::CRAWLER);
        let identity = client.validate(&header).unwrap();
        assert_eq!(identity.client_id, ClientId::Crawler);
        assert!(identity.permissions.contains(&Permission::TestPermission));
        assert!(identity.roles.contains(&Role::TestRole));
    }

    #[test]
    fn validate_valid_token_lowercase_bearer() {
        let client = AuthClient::new();
        let header = format!("bearer {}", model::tokens::VENDOR_SOURCING);
        let identity = client.validate(&header).unwrap();
        assert_eq!(identity.client_id, ClientId::VendorSourcing);
    }

    #[test]
    fn validate_invalid_token() {
        let client = AuthClient::new();
        let result = client.validate("Bearer invalid-token");
        assert!(matches!(result, Err(AuthError::InvalidToken)));
    }

    #[test]
    fn validate_missing_bearer_prefix() {
        let client = AuthClient::new();
        let result = client.validate(model::tokens::CRAWLER);
        assert!(matches!(result, Err(AuthError::InvalidFormat)));
    }

    #[test]
    fn validate_empty_token() {
        let client = AuthClient::new();
        let result = client.validate("Bearer ");
        assert!(matches!(result, Err(AuthError::InvalidFormat)));
    }

    #[test]
    fn validate_token_directly() {
        let client = AuthClient::new();
        let identity = client.validate_token(model::tokens::RETRIEVER).unwrap();
        assert_eq!(identity.client_id, ClientId::Retriever);
    }

    #[test]
    fn validate_bearer_with_extra_whitespace() {
        let client = AuthClient::new();

        // Multiple spaces after Bearer - the extra space becomes part of the token
        // which then gets trimmed
        let header = format!("Bearer  {}", model::tokens::VENDOR_SOURCING);
        let identity = client.validate(&header).unwrap();
        assert_eq!(identity.client_id, ClientId::VendorSourcing);

        // Trailing whitespace on token gets trimmed
        let header = format!("Bearer {}  ", model::tokens::VENDOR_SOURCING);
        let identity = client.validate(&header).unwrap();
        assert_eq!(identity.client_id, ClientId::VendorSourcing);

        // Leading and trailing whitespace on token gets trimmed
        let header = format!("Bearer   {}   ", model::tokens::VENDOR_SOURCING);
        let identity = client.validate(&header).unwrap();
        assert_eq!(identity.client_id, ClientId::VendorSourcing);
    }

    #[test]
    fn validate_mixed_case_bearer_prefix() {
        let client = AuthClient::new();

        // Mixed case "BeArEr" should be accepted (case-insensitive matching)
        let header = format!("BeArEr {}", model::tokens::VENDOR_SOURCING);
        let identity = client.validate(&header).unwrap();
        assert_eq!(identity.client_id, ClientId::VendorSourcing);

        // All uppercase "BEARER" should be accepted
        let header = format!("BEARER {}", model::tokens::RETRIEVER);
        let identity = client.validate(&header).unwrap();
        assert_eq!(identity.client_id, ClientId::Retriever);
    }

    #[test]
    fn validate_empty_authorization_header() {
        let client = AuthClient::new();

        // Empty string should return InvalidFormat
        let result = client.validate("");
        assert!(matches!(result, Err(AuthError::InvalidFormat)));

        // Whitespace-only should also return InvalidFormat
        let result = client.validate("   ");
        assert!(matches!(result, Err(AuthError::InvalidFormat)));
    }
}
