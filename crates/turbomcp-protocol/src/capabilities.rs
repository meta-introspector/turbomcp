//! # Capability Negotiation
//!
//! This module provides sophisticated capability negotiation and feature detection
//! for MCP protocol implementations.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::types::{ClientCapabilities, ServerCapabilities};

/// Capability matcher for negotiating features between client and server
#[derive(Debug, Clone)]
pub struct CapabilityMatcher {
    /// Feature compatibility rules
    compatibility_rules: HashMap<String, CompatibilityRule>,
    /// Default feature states
    defaults: HashMap<String, bool>,
}

/// Compatibility rule for a feature
#[derive(Debug, Clone)]
pub enum CompatibilityRule {
    /// Feature requires both client and server support
    RequireBoth,
    /// Feature requires only client support
    RequireClient,
    /// Feature requires only server support  
    RequireServer,
    /// Feature is optional (either side can enable)
    Optional,
    /// Custom compatibility function
    Custom(fn(&ClientCapabilities, &ServerCapabilities) -> bool),
}

/// Negotiated capability set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitySet {
    /// Enabled features
    pub enabled_features: HashSet<String>,
    /// Negotiated client capabilities
    pub client_capabilities: ClientCapabilities,
    /// Negotiated server capabilities
    pub server_capabilities: ServerCapabilities,
    /// Additional metadata from negotiation
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Capability negotiator for handling the negotiation process
#[derive(Debug, Clone)]
pub struct CapabilityNegotiator {
    /// Capability matcher
    matcher: CapabilityMatcher,
    /// Strict mode (fail on incompatible features)
    strict_mode: bool,
}

impl Default for CapabilityMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl CapabilityMatcher {
    /// Create a new capability matcher with default MCP rules
    pub fn new() -> Self {
        let mut matcher = Self {
            compatibility_rules: HashMap::new(),
            defaults: HashMap::new(),
        };

        // Set up default MCP capability rules
        matcher.add_rule("tools", CompatibilityRule::RequireServer);
        matcher.add_rule("prompts", CompatibilityRule::RequireServer);
        matcher.add_rule("resources", CompatibilityRule::RequireServer);
        matcher.add_rule("logging", CompatibilityRule::RequireServer);
        matcher.add_rule("sampling", CompatibilityRule::RequireClient);
        matcher.add_rule("roots", CompatibilityRule::RequireClient);
        matcher.add_rule("progress", CompatibilityRule::Optional);

        // Set defaults
        matcher.set_default("progress", true);

        matcher
    }

    /// Add a compatibility rule for a feature
    pub fn add_rule(&mut self, feature: &str, rule: CompatibilityRule) {
        self.compatibility_rules.insert(feature.to_string(), rule);
    }

    /// Set default state for a feature
    pub fn set_default(&mut self, feature: &str, enabled: bool) {
        self.defaults.insert(feature.to_string(), enabled);
    }

    /// Check if a feature is compatible between client and server
    pub fn is_compatible(
        &self,
        feature: &str,
        client: &ClientCapabilities,
        server: &ServerCapabilities,
    ) -> bool {
        self.compatibility_rules.get(feature).map_or_else(
            || {
                // Unknown feature - check if either side supports it
                Self::client_has_feature(feature, client)
                    || Self::server_has_feature(feature, server)
            },
            |rule| match rule {
                CompatibilityRule::RequireBoth => {
                    Self::client_has_feature(feature, client)
                        && Self::server_has_feature(feature, server)
                }
                CompatibilityRule::RequireClient => Self::client_has_feature(feature, client),
                CompatibilityRule::RequireServer => Self::server_has_feature(feature, server),
                CompatibilityRule::Optional => true,
                CompatibilityRule::Custom(func) => func(client, server),
            },
        )
    }

    /// Check if client has a specific feature
    fn client_has_feature(feature: &str, client: &ClientCapabilities) -> bool {
        match feature {
            "sampling" => client.sampling.is_some(),
            "roots" => client.roots.is_some(),
            _ => {
                // Check experimental features
                client
                    .experimental
                    .as_ref()
                    .is_some_and(|experimental| experimental.contains_key(feature))
            }
        }
    }

    /// Check if server has a specific feature
    fn server_has_feature(feature: &str, server: &ServerCapabilities) -> bool {
        match feature {
            "tools" => server.tools.is_some(),
            "prompts" => server.prompts.is_some(),
            "resources" => server.resources.is_some(),
            "logging" => server.logging.is_some(),
            _ => {
                // Check experimental features
                server
                    .experimental
                    .as_ref()
                    .is_some_and(|experimental| experimental.contains_key(feature))
            }
        }
    }

    /// Get all features from both client and server
    fn get_all_features(
        &self,
        client: &ClientCapabilities,
        server: &ServerCapabilities,
    ) -> HashSet<String> {
        let mut features = HashSet::new();

        // Standard client features
        if client.sampling.is_some() {
            features.insert("sampling".to_string());
        }
        if client.roots.is_some() {
            features.insert("roots".to_string());
        }

        // Standard server features
        if server.tools.is_some() {
            features.insert("tools".to_string());
        }
        if server.prompts.is_some() {
            features.insert("prompts".to_string());
        }
        if server.resources.is_some() {
            features.insert("resources".to_string());
        }
        if server.logging.is_some() {
            features.insert("logging".to_string());
        }

        // Experimental features
        if let Some(experimental) = &client.experimental {
            features.extend(experimental.keys().cloned());
        }
        if let Some(experimental) = &server.experimental {
            features.extend(experimental.keys().cloned());
        }

        // Add default features
        features.extend(self.defaults.keys().cloned());

        features
    }

    /// Negotiate capabilities between client and server
    pub fn negotiate(
        &self,
        client: &ClientCapabilities,
        server: &ServerCapabilities,
    ) -> Result<CapabilitySet, CapabilityError> {
        let all_features = self.get_all_features(client, server);
        let mut enabled_features = HashSet::new();
        let mut incompatible_features = Vec::new();

        for feature in &all_features {
            if self.is_compatible(feature, client, server) {
                enabled_features.insert(feature.clone());
            } else {
                incompatible_features.push(feature.clone());
            }
        }

        if !incompatible_features.is_empty() {
            return Err(CapabilityError::IncompatibleFeatures(incompatible_features));
        }

        // Apply defaults for features not explicitly enabled
        for (feature, enabled) in &self.defaults {
            if *enabled && !enabled_features.contains(feature) && all_features.contains(feature) {
                enabled_features.insert(feature.clone());
            }
        }

        Ok(CapabilitySet {
            enabled_features,
            client_capabilities: client.clone(),
            server_capabilities: server.clone(),
            metadata: HashMap::new(),
        })
    }
}

impl CapabilityNegotiator {
    /// Create a new capability negotiator
    pub const fn new(matcher: CapabilityMatcher) -> Self {
        Self {
            matcher,
            strict_mode: false,
        }
    }

    /// Enable strict mode (fail on any incompatible feature)
    pub const fn with_strict_mode(mut self) -> Self {
        self.strict_mode = true;
        self
    }

    /// Negotiate capabilities between client and server
    pub fn negotiate(
        &self,
        client: &ClientCapabilities,
        server: &ServerCapabilities,
    ) -> Result<CapabilitySet, CapabilityError> {
        match self.matcher.negotiate(client, server) {
            Ok(capability_set) => Ok(capability_set),
            Err(CapabilityError::IncompatibleFeatures(features)) if !self.strict_mode => {
                // In non-strict mode, just log the incompatible features and continue
                tracing::warn!(
                    "Some features are incompatible and will be disabled: {:?}",
                    features
                );

                // Create a capability set with only compatible features
                let all_features = self.matcher.get_all_features(client, server);
                let mut enabled_features = HashSet::new();

                for feature in &all_features {
                    if self.matcher.is_compatible(feature, client, server) {
                        enabled_features.insert(feature.clone());
                    }
                }

                Ok(CapabilitySet {
                    enabled_features,
                    client_capabilities: client.clone(),
                    server_capabilities: server.clone(),
                    metadata: HashMap::new(),
                })
            }
            Err(err) => Err(err),
        }
    }

    /// Check if a specific feature is enabled in the capability set
    pub fn is_feature_enabled(capability_set: &CapabilitySet, feature: &str) -> bool {
        capability_set.enabled_features.contains(feature)
    }

    /// Get all enabled features as a sorted vector
    pub fn get_enabled_features(capability_set: &CapabilitySet) -> Vec<String> {
        let mut features: Vec<String> = capability_set.enabled_features.iter().cloned().collect();
        features.sort();
        features
    }
}

impl Default for CapabilityNegotiator {
    fn default() -> Self {
        Self::new(CapabilityMatcher::new())
    }
}

impl CapabilitySet {
    /// Create a new empty capability set
    pub fn empty() -> Self {
        Self {
            enabled_features: HashSet::new(),
            client_capabilities: ClientCapabilities::default(),
            server_capabilities: ServerCapabilities::default(),
            metadata: HashMap::new(),
        }
    }

    /// Check if a feature is enabled
    pub fn has_feature(&self, feature: &str) -> bool {
        self.enabled_features.contains(feature)
    }

    /// Add a feature to the enabled set
    pub fn enable_feature(&mut self, feature: String) {
        self.enabled_features.insert(feature);
    }

    /// Remove a feature from the enabled set
    pub fn disable_feature(&mut self, feature: &str) {
        self.enabled_features.remove(feature);
    }

    /// Get the number of enabled features
    pub fn feature_count(&self) -> usize {
        self.enabled_features.len()
    }

    /// Add metadata
    pub fn add_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
    }

    /// Get metadata
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// Create a summary of enabled capabilities
    pub fn summary(&self) -> CapabilitySummary {
        CapabilitySummary {
            total_features: self.enabled_features.len(),
            client_features: self.count_client_features(),
            server_features: self.count_server_features(),
            enabled_features: self.enabled_features.iter().cloned().collect(),
        }
    }

    fn count_client_features(&self) -> usize {
        let mut count = 0;
        if self.client_capabilities.sampling.is_some() {
            count += 1;
        }
        if self.client_capabilities.roots.is_some() {
            count += 1;
        }
        if let Some(experimental) = &self.client_capabilities.experimental {
            count += experimental.len();
        }
        count
    }

    fn count_server_features(&self) -> usize {
        let mut count = 0;
        if self.server_capabilities.tools.is_some() {
            count += 1;
        }
        if self.server_capabilities.prompts.is_some() {
            count += 1;
        }
        if self.server_capabilities.resources.is_some() {
            count += 1;
        }
        if self.server_capabilities.logging.is_some() {
            count += 1;
        }
        if let Some(experimental) = &self.server_capabilities.experimental {
            count += experimental.len();
        }
        count
    }
}

/// Capability negotiation errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum CapabilityError {
    /// Features are incompatible between client and server
    #[error("Incompatible features: {0:?}")]
    IncompatibleFeatures(Vec<String>),
    /// Required feature is missing
    #[error("Required feature missing: {0}")]
    RequiredFeatureMissing(String),
    /// Protocol version mismatch
    #[error("Protocol version mismatch: client={client}, server={server}")]
    VersionMismatch {
        /// Client version string
        client: String,
        /// Server version string
        server: String,
    },
    /// Capability negotiation failed
    #[error("Capability negotiation failed: {0}")]
    NegotiationFailed(String),
}

/// Summary of capability negotiation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitySummary {
    /// Total number of enabled features
    pub total_features: usize,
    /// Number of client-side features
    pub client_features: usize,
    /// Number of server-side features
    pub server_features: usize,
    /// List of enabled features
    pub enabled_features: Vec<String>,
}

/// Utility functions for capability management
pub mod utils {
    use super::*;

    /// Create a minimal client capability set
    pub fn minimal_client_capabilities() -> ClientCapabilities {
        ClientCapabilities::default()
    }

    /// Create a minimal server capability set
    pub fn minimal_server_capabilities() -> ServerCapabilities {
        ServerCapabilities::default()
    }

    /// Create a full-featured client capability set
    pub fn full_client_capabilities() -> ClientCapabilities {
        ClientCapabilities {
            sampling: Some(Default::default()),
            roots: Some(Default::default()),
            elicitation: Some(Default::default()),
            experimental: None,
        }
    }

    /// Create a full-featured server capability set
    pub fn full_server_capabilities() -> ServerCapabilities {
        ServerCapabilities {
            tools: Some(Default::default()),
            prompts: Some(Default::default()),
            resources: Some(Default::default()),
            completions: Some(Default::default()),
            logging: Some(Default::default()),
            experimental: None,
        }
    }

    /// Check if two capability sets are compatible
    pub fn are_compatible(client: &ClientCapabilities, server: &ServerCapabilities) -> bool {
        let matcher = CapabilityMatcher::new();
        matcher.negotiate(client, server).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    #[test]
    fn test_capability_matcher() {
        let matcher = CapabilityMatcher::new();

        let client = ClientCapabilities {
            sampling: Some(SamplingCapabilities),
            roots: None,
            elicitation: None,
            experimental: None,
        };

        let server = ServerCapabilities {
            tools: Some(ToolsCapabilities::default()),
            prompts: None,
            resources: None,
            logging: None,
            completions: None,
            experimental: None,
        };

        assert!(matcher.is_compatible("sampling", &client, &server));
        assert!(matcher.is_compatible("tools", &client, &server));
        assert!(!matcher.is_compatible("roots", &client, &server));
    }

    #[test]
    fn test_capability_negotiation() {
        let negotiator = CapabilityNegotiator::default();

        let client = utils::full_client_capabilities();
        let server = utils::full_server_capabilities();

        let result = negotiator.negotiate(&client, &server);
        assert!(result.is_ok());

        let capability_set = result.unwrap();
        assert!(capability_set.has_feature("sampling"));
        assert!(capability_set.has_feature("tools"));
        assert!(capability_set.has_feature("roots"));
    }

    #[test]
    fn test_strict_mode() {
        let negotiator = CapabilityNegotiator::default().with_strict_mode();

        let client = ClientCapabilities::default();
        let server = ServerCapabilities::default();

        let result = negotiator.negotiate(&client, &server);
        assert!(result.is_ok()); // Should still work with minimal capabilities
    }

    #[test]
    fn test_capability_summary() {
        let mut capability_set = CapabilitySet::empty();
        capability_set.enable_feature("tools".to_string());
        capability_set.enable_feature("prompts".to_string());

        let summary = capability_set.summary();
        assert_eq!(summary.total_features, 2);
        assert!(summary.enabled_features.contains(&"tools".to_string()));
    }
}
