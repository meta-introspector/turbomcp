//! DPoP integration for transport layer
//!
//! This module provides seamless DPoP integration with the TurboMCP transport layer,
//! enabling automatic DPoP proof attachment and validation for HTTP-based transports.

use std::collections::HashMap;

use crate::core::{TransportError, TransportMessage, TransportMessageMetadata, TransportResult};

// Re-export DPoP types for convenience when using this integration
#[cfg(feature = "dpop")]
pub use turbomcp_dpop::{DpopError, DpopProof, DpopValidationResult};

/// DPoP integration extensions for TransportMessage
pub trait DpopTransportExt {
    /// Add a DPoP proof to the message headers
    fn with_dpop_proof(self, proof: &DpopProof) -> Self;

    /// Extract DPoP proof from message headers
    fn extract_dpop_proof(&self) -> TransportResult<Option<DpopProof>>;

    /// Check if message contains a DPoP proof
    fn has_dpop_proof(&self) -> bool;

    /// Validate DPoP proof for specific HTTP method and URI
    #[cfg(feature = "dpop")]
    fn validate_dpop_proof(
        &self,
        method: &str,
        uri: &str,
        access_token: Option<&str>,
    ) -> TransportResult<Option<DpopValidationResult>>;
}

impl DpopTransportExt for TransportMessage {
    fn with_dpop_proof(mut self, proof: &DpopProof) -> Self {
        // Add DPoP header as specified in RFC 9449
        self.metadata
            .headers
            .insert("DPoP".to_string(), proof.to_jwt_string());

        // Also add to custom headers for non-HTTP transports
        self.metadata
            .headers
            .insert("X-DPoP-Proof".to_string(), proof.to_jwt_string());

        self
    }

    fn extract_dpop_proof(&self) -> TransportResult<Option<DpopProof>> {
        // Try standard DPoP header first
        if let Some(dpop_header) = self.metadata.headers.get("DPoP") {
            return parse_dpop_header(dpop_header);
        }

        // Fallback to custom header for non-HTTP transports
        if let Some(dpop_header) = self.metadata.headers.get("X-DPoP-Proof") {
            return parse_dpop_header(dpop_header);
        }

        Ok(None)
    }

    fn has_dpop_proof(&self) -> bool {
        self.metadata.headers.contains_key("DPoP")
            || self.metadata.headers.contains_key("X-DPoP-Proof")
    }

    #[cfg(feature = "dpop")]
    fn validate_dpop_proof(
        &self,
        method: &str,
        uri: &str,
        access_token: Option<&str>,
    ) -> TransportResult<Option<DpopValidationResult>> {
        use std::sync::Arc;
        use turbomcp_dpop::{DpopKeyManager, DpopProofGenerator};

        // Extract DPoP proof from message
        let proof = match self.extract_dpop_proof()? {
            Some(proof) => proof,
            None => return Ok(None),
        };

        // Create a temporary validator (in production, this would be injected)
        // TODO: This should be provided by the transport layer configuration
        let key_manager = Arc::new(
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(async { DpopKeyManager::new_memory().await })
            })
            .map_err(|e| {
                TransportError::Internal(format!("DPoP key manager creation failed: {e}"))
            })?,
        );

        let validator = DpopProofGenerator::new(key_manager);

        // Perform async validation in a blocking context
        let result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                validator
                    .validate_proof(&proof, method, uri, access_token)
                    .await
            })
        })
        .map_err(|e| match e {
            turbomcp_dpop::DpopError::ProofValidationFailed { reason } => {
                TransportError::AuthenticationFailed(format!("DPoP validation failed: {reason}"))
            }
            turbomcp_dpop::DpopError::ReplayAttackDetected { nonce } => {
                TransportError::AuthenticationFailed(format!("DPoP replay attack: {nonce}"))
            }
            turbomcp_dpop::DpopError::ThumbprintMismatch { expected, actual } => {
                TransportError::AuthenticationFailed(format!(
                    "DPoP thumbprint mismatch: expected {expected}, got {actual}"
                ))
            }
            other => TransportError::AuthenticationFailed(format!("DPoP error: {other}")),
        })?;

        Ok(Some(result))
    }
}

/// Parse DPoP header value into DPoP proof structure
fn parse_dpop_header(header_value: &str) -> TransportResult<Option<DpopProof>> {
    // For now, we'll implement a simple parser
    // In the full implementation, this would parse the actual JWT
    if header_value.starts_with("dpop.proof.") {
        // This is a placeholder - the real implementation would decode the JWT
        // and reconstruct the DpopProof structure
        return Err(TransportError::Internal(
            "DPoP proof parsing not yet fully implemented".to_string(),
        ));
    }

    Ok(None)
}

/// DPoP-aware transport message metadata extensions
pub trait DpopMetadataExt {
    /// Add DPoP-specific metadata to the message
    fn with_dpop_metadata(self, thumbprint: String, algorithm: &str) -> Self;

    /// Get DPoP thumbprint from metadata
    fn dpop_thumbprint(&self) -> Option<&str>;

    /// Get DPoP algorithm from metadata  
    fn dpop_algorithm(&self) -> Option<&str>;
}

impl DpopMetadataExt for TransportMessageMetadata {
    fn with_dpop_metadata(mut self, thumbprint: String, algorithm: &str) -> Self {
        self.headers
            .insert("X-DPoP-Thumbprint".to_string(), thumbprint);
        self.headers
            .insert("X-DPoP-Algorithm".to_string(), algorithm.to_string());
        self
    }

    fn dpop_thumbprint(&self) -> Option<&str> {
        self.headers.get("X-DPoP-Thumbprint").map(String::as_str)
    }

    fn dpop_algorithm(&self) -> Option<&str> {
        self.headers.get("X-DPoP-Algorithm").map(String::as_str)
    }
}

/// Builder pattern for creating DPoP-enabled transport messages
#[derive(Debug, Default)]
pub struct DpopMessageBuilder {
    headers: HashMap<String, String>,
    content_type: Option<String>,
    correlation_id: Option<String>,
    dpop_proof: Option<String>,
    dpop_metadata: Option<(String, String)>, // (thumbprint, algorithm)
}

impl DpopMessageBuilder {
    /// Create a new DPoP message builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a custom header
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set content type
    pub fn content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }

    /// Set correlation ID
    pub fn correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }

    /// Add DPoP proof
    pub fn dpop_proof(mut self, proof: &DpopProof) -> Self {
        self.dpop_proof = Some(proof.to_jwt_string());
        self
    }

    /// Add DPoP metadata
    pub fn dpop_metadata(mut self, thumbprint: String, algorithm: String) -> Self {
        self.dpop_metadata = Some((thumbprint, algorithm));
        self
    }

    /// Build the transport message metadata
    pub fn build_metadata(self) -> TransportMessageMetadata {
        let mut metadata = TransportMessageMetadata {
            content_type: self.content_type,
            correlation_id: self.correlation_id,
            headers: self.headers,
            ..Default::default()
        };

        // Add DPoP proof if provided
        if let Some(proof) = self.dpop_proof {
            metadata.headers.insert("DPoP".to_string(), proof.clone());
            metadata.headers.insert("X-DPoP-Proof".to_string(), proof);
        }

        // Add DPoP metadata if provided
        if let Some((thumbprint, algorithm)) = self.dpop_metadata {
            metadata = metadata.with_dpop_metadata(thumbprint, &algorithm);
        }

        metadata
    }

    /// Build a complete transport message
    pub fn build_message(
        self,
        id: turbomcp_core::MessageId,
        payload: bytes::Bytes,
    ) -> TransportMessage {
        TransportMessage::with_metadata(id, payload, self.build_metadata())
    }
}

/// DPoP configuration for transport layers
#[derive(Debug, Clone)]
pub struct DpopTransportConfig {
    /// Whether DPoP is enabled for this transport
    pub enabled: bool,
    /// Whether to require DPoP for all requests
    pub required: bool,
    /// Allowed clock skew in seconds
    pub clock_skew_tolerance: u64,
    /// DPoP proof lifetime in seconds
    pub proof_lifetime: u64,
    /// Whether to validate DPoP proofs automatically
    pub auto_validate: bool,
}

impl Default for DpopTransportConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            required: false,
            clock_skew_tolerance: 300, // 5 minutes
            proof_lifetime: 60,        // 1 minute
            auto_validate: true,
        }
    }
}

impl DpopTransportConfig {
    /// Create a development configuration
    pub fn development() -> Self {
        Self {
            enabled: true,
            required: false,
            clock_skew_tolerance: 600, // 10 minutes for development
            proof_lifetime: 300,       // 5 minutes for development
            auto_validate: true,
        }
    }

    /// Create a production configuration
    pub fn production() -> Self {
        Self {
            enabled: true,
            required: true,
            clock_skew_tolerance: 60, // 1 minute for production
            proof_lifetime: 60,       // 1 minute for production
            auto_validate: true,
        }
    }

    /// Create a high-security configuration
    pub fn high_security() -> Self {
        Self {
            enabled: true,
            required: true,
            clock_skew_tolerance: 30, // 30 seconds
            proof_lifetime: 30,       // 30 seconds
            auto_validate: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use turbomcp_core::MessageId;

    #[test]
    fn test_dpop_message_builder() {
        let builder = DpopMessageBuilder::new()
            .header("Authorization", "Bearer token123")
            .content_type("application/json")
            .correlation_id("req-123")
            .dpop_metadata("thumb123".to_string(), "ES256".to_string());

        let metadata = builder.build_metadata();

        assert_eq!(metadata.content_type, Some("application/json".to_string()));
        assert_eq!(metadata.correlation_id, Some("req-123".to_string()));
        assert_eq!(
            metadata.headers.get("Authorization"),
            Some(&"Bearer token123".to_string())
        );
        assert_eq!(metadata.dpop_thumbprint(), Some("thumb123"));
        assert_eq!(metadata.dpop_algorithm(), Some("ES256"));
    }

    #[test]
    fn test_transport_message_dpop_integration() {
        let message =
            TransportMessage::new(MessageId::from("test-123"), Bytes::from("test payload"));

        // Initially should not have DPoP proof
        assert!(!message.has_dpop_proof());

        // Test that we can check for DPoP without panicking
        // (Full implementation would require actual DPoP proof)
        let extract_result = message.extract_dpop_proof();
        assert!(extract_result.is_ok());
        assert!(extract_result.unwrap().is_none());
    }

    #[test]
    fn test_dpop_transport_config() {
        let dev_config = DpopTransportConfig::development();
        assert!(dev_config.enabled);
        assert!(!dev_config.required);
        assert_eq!(dev_config.clock_skew_tolerance, 600);

        let prod_config = DpopTransportConfig::production();
        assert!(prod_config.enabled);
        assert!(prod_config.required);
        assert_eq!(prod_config.clock_skew_tolerance, 60);

        let secure_config = DpopTransportConfig::high_security();
        assert!(secure_config.enabled);
        assert!(secure_config.required);
        assert_eq!(secure_config.clock_skew_tolerance, 30);
        assert_eq!(secure_config.proof_lifetime, 30);
    }
}
