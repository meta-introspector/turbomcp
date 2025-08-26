//! Comprehensive tests for turbomcp-client lib.rs

use async_trait::async_trait;
use turbomcp_client::{
    Client, ClientBuilder, ClientCapabilities, InitializeResult,
    PublicServerCapabilities as ServerCapabilities,
};
use turbomcp_transport::core::{
    Transport, TransportCapabilities, TransportMessage, TransportMetrics, TransportResult,
    TransportState, TransportType,
};

// Mock transport that implements the Transport trait
#[derive(Debug)]
struct MockTransport {
    capabilities: TransportCapabilities,
    state: TransportState,
    metrics: TransportMetrics,
}

impl MockTransport {
    fn new() -> Self {
        Self {
            capabilities: TransportCapabilities::default(),
            state: TransportState::Disconnected,
            metrics: TransportMetrics::default(),
        }
    }
}

#[async_trait]
impl Transport for MockTransport {
    fn transport_type(&self) -> TransportType {
        TransportType::Stdio
    }

    fn capabilities(&self) -> &TransportCapabilities {
        &self.capabilities
    }

    async fn state(&self) -> TransportState {
        self.state.clone()
    }

    async fn connect(&mut self) -> TransportResult<()> {
        self.state = TransportState::Connected;
        Ok(())
    }

    async fn disconnect(&mut self) -> TransportResult<()> {
        self.state = TransportState::Disconnected;
        Ok(())
    }

    async fn send(&mut self, _message: TransportMessage) -> TransportResult<()> {
        Ok(())
    }

    async fn receive(&mut self) -> TransportResult<Option<TransportMessage>> {
        Ok(None)
    }

    async fn metrics(&self) -> TransportMetrics {
        self.metrics.clone()
    }

    fn endpoint(&self) -> Option<String> {
        Some("mock://transport".to_string())
    }
}

// ClientCapabilities tests
#[test]
fn test_client_capabilities_new() {
    let capabilities = ClientCapabilities::default();
    assert!(!capabilities.tools);
}

#[test]
fn test_client_capabilities_debug() {
    let capabilities = ClientCapabilities {
        tools: true,
        prompts: false,
        resources: false,
        sampling: false,
    };
    let debug_str = format!("{capabilities:?}");
    assert!(debug_str.contains("ClientCapabilities"));
    assert!(debug_str.contains("tools: true"));
}

#[test]
fn test_client_capabilities_clone() {
    let original = ClientCapabilities {
        tools: true,
        prompts: false,
        resources: false,
        sampling: false,
    };
    let cloned = original.clone();
    assert_eq!(original.tools, cloned.tools);
}

#[test]
fn test_client_capabilities_custom_values() {
    let capabilities = ClientCapabilities {
        tools: true,
        prompts: false,
        resources: false,
        sampling: false,
    };
    assert!(capabilities.tools);

    let no_capabilities = ClientCapabilities {
        tools: false,
        prompts: false,
        resources: false,
        sampling: false,
    };
    assert!(!no_capabilities.tools);
}

// Client tests (without async since the current implementation is sync)
#[test]
fn test_client_new() {
    let transport = MockTransport::new();
    let client = Client::new(transport);

    let debug_str = format!("{client:?}");
    assert!(debug_str.contains("Client"));
}

#[test]
fn test_client_new_with_different_transports() {
    let transport1 = MockTransport::new();
    let transport2 = MockTransport::new();

    let client1 = Client::new(transport1);
    let client2 = Client::new(transport2);

    assert!(format!("{client1:?}").contains("Client"));
    assert!(format!("{client2:?}").contains("Client"));
}

// Since the methods are async, I'll test the structure without the async calls
#[test]
fn test_client_structure() {
    let transport = MockTransport::new();
    let client = Client::new(transport);

    // Test that the client can be created and has the expected structure
    let debug_str = format!("{client:?}");
    assert!(debug_str.contains("Client"));
}

// InitializeResult tests
#[test]
fn test_initialize_result_structure() {
    let server_info = turbomcp_protocol::Implementation {
        name: "test-server".to_string(),
        version: "2.0.0".to_string(),
        title: Some("Test Server".to_string()),
    };

    let result = InitializeResult {
        server_info,
        server_capabilities: ServerCapabilities {
            tools: None,
            prompts: None,
            resources: None,
            experimental: None,
            logging: None,
            completions: None,
        },
    };

    assert_eq!(result.server_info.name, "test-server");
    assert_eq!(result.server_info.version, "2.0.0");
    assert_eq!(result.server_info.title, Some("Test Server".to_string()));
}

#[test]
fn test_initialize_result_with_no_title() {
    let server_info = turbomcp_protocol::Implementation {
        name: "no-title-server".to_string(),
        version: "1.0.0".to_string(),
        title: None,
    };

    let result = InitializeResult {
        server_info,
        server_capabilities: ServerCapabilities {
            tools: None,
            prompts: None,
            resources: None,
            experimental: None,
            logging: None,
            completions: None,
        },
    };

    assert_eq!(result.server_info.name, "no-title-server");
    assert_eq!(result.server_info.version, "1.0.0");
    assert_eq!(result.server_info.title, None);
}

// ClientBuilder tests
#[test]
fn test_client_builder_new() {
    let builder = ClientBuilder::new();
    let debug_str = format!("{builder:?}");
    assert!(debug_str.contains("ClientBuilder"));
}

#[test]
fn test_client_builder_default() {
    let builder = ClientBuilder::new();
    let debug_str = format!("{builder:?}");
    assert!(debug_str.contains("ClientBuilder"));
}

#[test]
fn test_client_builder_multiple_instances() {
    let builder1 = ClientBuilder::new();
    let builder2 = ClientBuilder::new();

    // Both should create successfully
    assert!(format!("{builder1:?}").contains("ClientBuilder"));
    assert!(format!("{builder2:?}").contains("ClientBuilder"));
}

// Test client with mock transport
#[test]
fn test_client_with_mock_transport() {
    let transport = MockTransport::new();
    let client = Client::new(transport);
    assert!(format!("{client:?}").contains("Client"));
}

#[test]
fn test_client_with_different_transport_types() {
    let transports = vec![MockTransport::new(), MockTransport::new()];

    for transport in transports {
        let client = Client::new(transport);
        assert!(format!("{client:?}").contains("Client"));
    }
}

// Type compatibility tests
#[test]
fn test_client_generic_over_transport() {
    fn create_client<T: Transport>(transport: T) -> Client<T> {
        Client::new(transport)
    }

    let transport = MockTransport::new();
    let client = create_client(transport);
    assert!(format!("{client:?}").contains("Client"));
}

#[test]
fn test_client_capabilities_configuration() {
    let capabilities_configs = vec![
        ClientCapabilities {
            tools: false,
            prompts: false,
            resources: false,
            sampling: false,
        },
        ClientCapabilities {
            tools: true,
            prompts: false,
            resources: false,
            sampling: false,
        },
    ];

    for config in capabilities_configs {
        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("ClientCapabilities"));
    }
}

#[test]
fn test_builder_pattern_usage() {
    let builder = ClientBuilder::new();
    assert!(format!("{builder:?}").contains("ClientBuilder"));

    let default_builder = ClientBuilder::new();
    assert!(format!("{default_builder:?}").contains("ClientBuilder"));
}

// Edge case tests
#[test]
fn test_client_capabilities_edge_cases() {
    let all_false = ClientCapabilities {
        tools: false,
        prompts: false,
        resources: false,
        sampling: false,
    };
    assert!(!all_false.tools);

    let all_true = ClientCapabilities {
        tools: true,
        prompts: false,
        resources: false,
        sampling: false,
    };
    assert!(all_true.tools);
}

// Test multiple client instances
#[test]
fn test_multiple_client_instances() {
    let transport1 = MockTransport::new();
    let transport2 = MockTransport::new();

    let client1 = Client::new(transport1);
    let client2 = Client::new(transport2);

    // Both clients should be independent
    assert!(format!("{client1:?}").contains("Client"));
    assert!(format!("{client2:?}").contains("Client"));
}

// Test initialize result variants
#[test]
fn test_initialize_result_variants() {
    let implementations = vec![
        turbomcp_protocol::Implementation {
            name: "server1".to_string(),
            version: "1.0.0".to_string(),
            title: Some("Server 1".to_string()),
        },
        turbomcp_protocol::Implementation {
            name: "server2".to_string(),
            version: "2.0.0".to_string(),
            title: None,
        },
        turbomcp_protocol::Implementation {
            name: "minimal".to_string(),
            version: "0.1.0".to_string(),
            title: Some("Minimal Server".to_string()),
        },
    ];

    for server_info in implementations {
        let result = InitializeResult {
            server_info,
            server_capabilities: ServerCapabilities {
                tools: None,
                prompts: None,
                resources: None,
                experimental: None,
                logging: None,
                completions: None,
            },
        };
        assert!(!result.server_info.name.is_empty());
        assert!(!result.server_info.version.is_empty());
    }
}

// Test client capabilities with different configurations
#[test]
fn test_client_capabilities_serialization() {
    let capabilities = ClientCapabilities {
        tools: true,
        prompts: false,
        resources: false,
        sampling: false,
    };

    // Test that capabilities can be formatted for debug
    let debug_output = format!("{capabilities:?}");
    assert!(debug_output.contains("tools"));
    assert!(debug_output.contains("true"));
}

// Complete workflow test (without async operations)
#[test]
fn test_complete_client_setup_pattern() {
    // Step 1: Create capabilities
    let capabilities = ClientCapabilities::default();
    assert!(!capabilities.tools);

    // Step 2: Create transport
    let transport = MockTransport::new();

    // Step 3: Create client
    let client = Client::new(transport);
    assert!(format!("{client:?}").contains("Client"));

    // Step 4: Create builder
    let builder = ClientBuilder::new();
    assert!(format!("{builder:?}").contains("ClientBuilder"));
}

// Integration test
#[test]
fn test_client_library_integration() {
    // Test that all the main types work together
    let capabilities = ClientCapabilities {
        tools: true,
        prompts: false,
        resources: false,
        sampling: false,
    };
    let transport = MockTransport::new();
    let client = Client::new(transport);
    let builder = ClientBuilder::new();

    // All should be successfully created
    assert!(capabilities.tools);
    assert!(format!("{client:?}").contains("Client"));
    assert!(format!("{builder:?}").contains("ClientBuilder"));

    // Test InitializeResult
    let server_info = turbomcp_protocol::Implementation {
        name: "integration-server".to_string(),
        version: "1.0.0".to_string(),
        title: Some("Integration Test Server".to_string()),
    };
    let init_result = InitializeResult {
        server_info,
        server_capabilities: ServerCapabilities {
            tools: None,
            prompts: None,
            resources: None,
            experimental: None,
            logging: None,
            completions: None,
        },
    };
    assert_eq!(init_result.server_info.name, "integration-server");
}

// Test edge cases and error conditions
#[test]
fn test_client_edge_cases() {
    // Test with empty names
    let server_info = turbomcp_protocol::Implementation {
        name: "".to_string(),
        version: "".to_string(),
        title: Some("".to_string()),
    };
    let result = InitializeResult {
        server_info,
        server_capabilities: ServerCapabilities {
            tools: None,
            prompts: None,
            resources: None,
            experimental: None,
            logging: None,
            completions: None,
        },
    };
    assert_eq!(result.server_info.name, "");
    assert_eq!(result.server_info.version, "");
    assert_eq!(result.server_info.title, Some("".to_string()));
}

// Test boundary conditions
#[test]
fn test_client_boundary_conditions() {
    // Test with very long strings
    let long_name = "a".repeat(1000);
    let long_version = "1.".repeat(500) + "0";

    let server_info = turbomcp_protocol::Implementation {
        name: long_name.clone(),
        version: long_version.clone(),
        title: Some("Long Title".to_string()),
    };

    let result = InitializeResult {
        server_info,
        server_capabilities: ServerCapabilities {
            tools: None,
            prompts: None,
            resources: None,
            experimental: None,
            logging: None,
            completions: None,
        },
    };
    assert_eq!(result.server_info.name, long_name);
    assert_eq!(result.server_info.version, long_version);
}
