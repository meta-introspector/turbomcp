//! Test utilities for DPoP implementation
//!
//! This module provides testing utilities and mock implementations
//! for DPoP components when the `test-utils` feature is enabled.

#[cfg(feature = "test-utils")]
use crate::{DpopAlgorithm, DpopKeyPair, Result};

/// Mock key manager for testing
#[cfg(feature = "test-utils")]
#[derive(Debug)]
pub struct MockKeyManager {
    _keys: Vec<DpopKeyPair>,
}

#[cfg(feature = "test-utils")]
impl MockKeyManager {
    /// Create a new mock key manager
    pub fn new() -> Self {
        Self { _keys: Vec::new() }
    }

    /// Generate a mock key pair for testing
    pub async fn generate_test_key(&self, _algorithm: DpopAlgorithm) -> Result<DpopKeyPair> {
        // Placeholder implementation
        Err(crate::DpopError::KeyManagementError {
            reason: "Mock key generation not implemented".to_string(),
        })
    }
}

#[cfg(feature = "test-utils")]
impl Default for MockKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

// Placeholder when feature is not enabled
#[cfg(not(feature = "test-utils"))]
#[derive(Debug)]
pub struct MockKeyManager;

#[cfg(not(feature = "test-utils"))]
impl MockKeyManager {
    /// Create a new mock key manager (feature disabled)
    pub fn new() -> Self {
        Self
    }
}
