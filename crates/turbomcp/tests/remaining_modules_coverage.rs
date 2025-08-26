//! Test coverage for server, context, and schema modules

use std::collections::HashMap;
use std::sync::Arc;

use turbomcp::prelude::*;
use turbomcp::server::*;

// Remove unused import - we now test actual macro-generated schemas

// Import context module types
use turbomcp::context::{
    Container, FactoryProvider, ServiceInfo, ServiceProvider, SingletonProvider,
};

/// Test server module - HandlerInfo and HandlerType
#[tokio::test]
async fn test_server_module_comprehensive() {
    // Test HandlerType enum
    assert_eq!(HandlerType::Tool, HandlerType::Tool);
    assert_eq!(HandlerType::Prompt, HandlerType::Prompt);
    assert_eq!(HandlerType::Resource, HandlerType::Resource);

    assert_ne!(HandlerType::Tool, HandlerType::Prompt);
    assert_ne!(HandlerType::Prompt, HandlerType::Resource);
    assert_ne!(HandlerType::Tool, HandlerType::Resource);

    // Test debug formatting
    assert_eq!(format!("{:?}", HandlerType::Tool), "Tool");
    assert_eq!(format!("{:?}", HandlerType::Prompt), "Prompt");
    assert_eq!(format!("{:?}", HandlerType::Resource), "Resource");

    // Test HandlerInfo creation
    let handler_info = HandlerInfo {
        name: "test_tool".to_string(),
        handler_type: HandlerType::Tool,
        description: Some("A test tool handler".to_string()),
        metadata: serde_json::json!({
            "version": "1.0.0",
            "author": "test",
            "tags": ["testing", "demo"]
        }),
    };

    assert_eq!(handler_info.name, "test_tool");
    assert_eq!(handler_info.handler_type, HandlerType::Tool);
    assert_eq!(
        handler_info.description,
        Some("A test tool handler".to_string())
    );
    assert!(handler_info.metadata.is_object());

    // Test cloning
    let cloned_info = handler_info.clone();
    assert_eq!(cloned_info.name, handler_info.name);
    assert_eq!(cloned_info.handler_type, handler_info.handler_type);
    assert_eq!(cloned_info.description, handler_info.description);

    // Test with different handler types
    let prompt_info = HandlerInfo {
        name: "test_prompt".to_string(),
        handler_type: HandlerType::Prompt,
        description: None,
        metadata: serde_json::json!({}),
    };

    assert_eq!(prompt_info.handler_type, HandlerType::Prompt);
    assert_eq!(prompt_info.description, None);

    let resource_info = HandlerInfo {
        name: "test_resource".to_string(),
        handler_type: HandlerType::Resource,
        description: Some("A resource handler".to_string()),
        metadata: serde_json::json!({"uri_template": "file://{path}"}),
    };

    assert_eq!(resource_info.handler_type, HandlerType::Resource);
    assert!(resource_info.metadata["uri_template"].is_string());
}

/// Test global handler registry
#[tokio::test]
async fn test_global_handler_registry() {
    // Clear registry first (in case other tests left data)
    let initial_handlers = get_registered_handlers();
    let initial_count = initial_handlers.len();

    // Register a handler
    let handler1 = HandlerInfo {
        name: "registry_test_1".to_string(),
        handler_type: HandlerType::Tool,
        description: Some("First test handler".to_string()),
        metadata: serde_json::json!({"priority": 1}),
    };

    register_handler(handler1.clone());

    // Check that it was registered
    let handlers = get_registered_handlers();
    assert_eq!(handlers.len(), initial_count + 1);

    let registered = handlers.iter().find(|h| h.name == "registry_test_1");
    assert!(registered.is_some());
    let registered = registered.unwrap();
    assert_eq!(registered.name, handler1.name);
    assert_eq!(registered.handler_type, handler1.handler_type);

    // Register multiple handlers
    let handler2 = HandlerInfo {
        name: "registry_test_2".to_string(),
        handler_type: HandlerType::Prompt,
        description: None,
        metadata: serde_json::json!({"priority": 2}),
    };

    let handler3 = HandlerInfo {
        name: "registry_test_3".to_string(),
        handler_type: HandlerType::Resource,
        description: Some("Third test handler".to_string()),
        metadata: serde_json::json!({"priority": 3}),
    };

    register_handler(handler2.clone());
    register_handler(handler3.clone());

    let handlers = get_registered_handlers();
    assert!(handlers.len() >= initial_count + 3);

    // Verify all handlers are present
    let names: Vec<_> = handlers.iter().map(|h| &h.name).collect();
    assert!(names.contains(&&"registry_test_1".to_string()));
    assert!(names.contains(&&"registry_test_2".to_string()));
    assert!(names.contains(&&"registry_test_3".to_string()));

    // Verify handler types are correct
    let tool_handler = handlers
        .iter()
        .find(|h| h.name == "registry_test_1")
        .unwrap();
    assert_eq!(tool_handler.handler_type, HandlerType::Tool);

    let prompt_handler = handlers
        .iter()
        .find(|h| h.name == "registry_test_2")
        .unwrap();
    assert_eq!(prompt_handler.handler_type, HandlerType::Prompt);

    let resource_handler = handlers
        .iter()
        .find(|h| h.name == "registry_test_3")
        .unwrap();
    assert_eq!(resource_handler.handler_type, HandlerType::Resource);
}

/// Test actual macro-generated schemas with feature enabled
#[cfg(feature = "schema-generation")]
#[tokio::test]
async fn test_schema_generation_with_feature() {
    // Test actual macro-generated schemas instead of schemars
    #[derive(Clone)]
    struct TestModulesServer;

    #[server]
    #[allow(dead_code, clippy::too_many_arguments)]
    impl TestModulesServer {
        #[tool("Simple data tool")]
        async fn process_simple_data(
            &self,
            name: String,
            count: u32,
            active: bool,
        ) -> McpResult<String> {
            Ok(format!(
                "Processing {} (count: {}, active: {})",
                name, count, active
            ))
        }

        #[tool("Complex nested data tool")]
        async fn process_person(
            &self,
            #[allow(unused_variables)] id: u64,
            name: String,
            #[allow(unused_variables)] email: Option<String>,
            #[allow(unused_variables)] street: String,
            city: String,
            country: String,
            hobbies: Vec<String>,
            #[allow(unused_variables)] metadata: std::collections::HashMap<
                String,
                serde_json::Value,
            >,
        ) -> McpResult<String> {
            Ok(format!(
                "Person {} from {}, {} with {} hobbies",
                name,
                city,
                country,
                hobbies.len()
            ))
        }

        #[tool("Status processing tool")]
        async fn process_status(
            &self,
            status: String, // Simplified from enum for now
            timestamp: String,
        ) -> McpResult<String> {
            Ok(format!("Status: {} at {}", status, timestamp))
        }

        #[tool("Type testing tool")]
        async fn test_various_types(
            &self,
            #[allow(unused_variables)] text: String,
            number: i32,
            items: Vec<String>,
            #[allow(unused_variables)] optional: Option<String>,
        ) -> McpResult<String> {
            Ok(format!("Got {} items, number: {}", items.len(), number))
        }
    }

    // Test actual macro-generated schema for simple data
    let (name, desc, schema) = TestModulesServer::process_simple_data_metadata();
    assert_eq!(name, "process_simple_data");
    assert!(!desc.is_empty());
    assert!(!schema.is_null());

    let properties = schema["properties"].as_object().unwrap();
    assert!(properties.contains_key("name"));
    assert!(properties.contains_key("count"));
    assert!(properties.contains_key("active"));

    // Test complex nested schema
    let (_, _, person_schema) = TestModulesServer::process_person_metadata();
    let person_properties = person_schema["properties"].as_object().unwrap();

    assert!(person_properties.contains_key("street"));
    assert!(person_properties.contains_key("hobbies"));
    assert!(person_properties.contains_key("metadata"));

    // Test status schema
    let (_, _, status_schema) = TestModulesServer::process_status_metadata();
    assert!(!status_schema.is_null());

    // Test various types schema - validates actual macro type inference
    let (_, _, types_schema) = TestModulesServer::test_various_types_metadata();
    let types_properties = types_schema["properties"].as_object().unwrap();

    // Verify type inference from Rust to JSON Schema
    assert_eq!(types_properties["text"]["type"], "string");
    assert_eq!(types_properties["number"]["type"], "integer");
    assert_eq!(types_properties["items"]["type"], "array");

    // Verify optional parameter handling
    let required = types_schema["required"].as_array().unwrap();
    assert!(required.contains(&serde_json::Value::String("text".to_string())));
    assert!(!required.contains(&serde_json::Value::String("optional".to_string())));
}

/// Test macro behavior without schema-generation feature
#[cfg(not(feature = "schema-generation"))]
#[tokio::test]
async fn test_schema_generation_without_feature() {
    // Test that macros still work without schema-generation feature
    #[derive(Clone)]
    struct NoSchemaServer;

    #[server]
    impl NoSchemaServer {
        #[tool("Basic tool without schema generation")]
        async fn basic_tool(&self, input: String, count: i32) -> McpResult<String> {
            Ok(format!("Processed {} with count {}", input, count))
        }
    }

    // Without the feature, schemas should still be generated but simpler
    let (name, desc, schema) = NoSchemaServer::basic_tool_metadata();
    assert_eq!(name, "basic_tool");
    assert!(!desc.is_empty());
    // Schema should exist even without feature, may be basic structure
    assert!(!schema.is_null());
}

/// Test dependency injection container - basic functionality
#[tokio::test]
async fn test_container_basic_functionality() {
    let container = Container::new();

    // Test registering a direct service
    container
        .register("config", "test_config_value".to_string())
        .await;

    // Test resolving the service
    let config: String = container.resolve("config").await.unwrap();
    assert_eq!(config, "test_config_value");

    // Test service not found
    let missing: Result<String, _> = container.resolve("missing_service").await;
    assert!(missing.is_err());

    // Test type mismatch
    let wrong_type: Result<i32, _> = container.resolve("config").await;
    assert!(wrong_type.is_err());

    // Test service info
    let info = container.get_service_info("config").await;
    assert!(info.is_some());
    let info = info.unwrap();
    assert_eq!(info.name, "config");
    assert!(!info.is_singleton);

    // Test has_service
    assert!(container.has_service("config").await);
    assert!(!container.has_service("missing").await);

    // Test list_services
    let services = container.list_services().await;
    assert!(services.iter().any(|s| s.name == "config"));
}

/// Test factory provider
#[tokio::test]
async fn test_factory_provider() {
    let container = Container::new();

    // Register a factory service
    let counter = Arc::new(std::sync::atomic::AtomicI32::new(0));
    let counter_clone = Arc::clone(&counter);

    container
        .register_factory("counter", move || {
            counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        })
        .await;

    // Resolve multiple times - should get different values (not singleton)
    let val1: i32 = container.resolve("counter").await.unwrap();
    let val2: i32 = container.resolve("counter").await.unwrap();
    let val3: i32 = container.resolve("counter").await.unwrap();

    assert_eq!(val1, 0); // fetch_add returns previous value
    assert_eq!(val2, 1); // fetch_add returns previous value
    assert_eq!(val3, 2); // fetch_add returns previous value

    // Test service info
    let info = container.get_service_info("counter").await.unwrap();
    assert_eq!(info.name, "counter");
    assert!(!info.is_singleton);
}

/// Test singleton provider
#[tokio::test]
async fn test_singleton_provider() {
    let container = Container::new();

    // Register a singleton service with a counter
    let counter = Arc::new(std::sync::atomic::AtomicI32::new(0));
    let counter_clone = Arc::clone(&counter);

    container
        .register_singleton("singleton_counter", move || {
            counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        })
        .await;

    // Resolve multiple times - should get same value (singleton behavior)
    let val1: i32 = container.resolve("singleton_counter").await.unwrap();
    let val2: i32 = container.resolve("singleton_counter").await.unwrap();
    let val3: i32 = container.resolve("singleton_counter").await.unwrap();

    assert_eq!(val1, 0); // fetch_add returns previous value, called only once for singleton
    assert_eq!(val2, 0); // Same value due to singleton behavior
    assert_eq!(val3, 0); // Same value due to singleton behavior

    // Test service info
    let info = container
        .get_service_info("singleton_counter")
        .await
        .unwrap();
    assert_eq!(info.name, "singleton_counter");
    assert!(info.is_singleton);
}

/// Test complex dependency injection scenarios
#[tokio::test]
async fn test_complex_dependency_injection() {
    #[derive(Clone)]
    struct DatabaseConnection {
        url: String,
    }

    #[derive(Clone)]
    struct ConfigService {
        settings: HashMap<String, String>,
    }

    #[derive(Clone)]
    #[allow(dead_code)]
    struct UserService {
        db: DatabaseConnection,
        config: ConfigService,
    }

    let container = Container::new();

    // Register basic services
    container
        .register("db_url", "postgres://localhost/test".to_string())
        .await;

    // Register complex services with dependencies
    container
        .register_factory("database", || DatabaseConnection {
            url: "postgres://localhost/test".to_string(),
        })
        .await;

    let mut settings = HashMap::new();
    settings.insert("max_connections".to_string(), "100".to_string());
    settings.insert("timeout".to_string(), "30".to_string());

    container
        .register("config_service", ConfigService { settings })
        .await;

    // Resolve services
    let db: DatabaseConnection = container.resolve("database").await.unwrap();
    assert_eq!(db.url, "postgres://localhost/test");

    let config: ConfigService = container.resolve("config_service").await.unwrap();
    assert_eq!(
        config.settings.get("max_connections"),
        Some(&"100".to_string())
    );

    // Test with dependency resolution
    let db_with_deps: DatabaseConnection = container
        .resolve_with_dependencies("database")
        .await
        .unwrap();
    assert_eq!(db_with_deps.url, "postgres://localhost/test");
}

/// Test circular dependency detection
#[tokio::test]
async fn test_circular_dependency_detection() {
    // Create a custom provider with dependencies to test circular dependency detection
    struct CustomProvider {
        deps: Vec<String>,
    }

    #[async_trait]
    impl ServiceProvider for CustomProvider {
        type Output = String;

        async fn provide(&self, _container: &Container) -> McpResult<Self::Output> {
            Ok("service".to_string())
        }

        fn dependencies(&self) -> Vec<String> {
            self.deps.clone()
        }
    }

    let container = Container::new();

    // First, test a simple case without circular dependencies
    let _provider_a = CustomProvider {
        deps: vec!["service_b".to_string()],
    };

    let _provider_b = CustomProvider {
        deps: vec![], // No dependencies
    };

    // This would require more complex setup to test actual circular dependencies
    // For now, test the basic resolution without circular deps
    container.register("service_b", "b_value".to_string()).await;

    let result: String = container
        .resolve_with_dependencies("service_b")
        .await
        .unwrap();
    assert_eq!(result, "b_value");
}

/// Test container edge cases
#[tokio::test]
async fn test_container_edge_cases() {
    let container = Container::new();

    // Test with empty string names
    container
        .register("", "empty_name_service".to_string())
        .await;
    let empty_name: String = container.resolve("").await.unwrap();
    assert_eq!(empty_name, "empty_name_service");

    // Test with unicode service names
    container
        .register("服务", "unicode_service".to_string())
        .await;
    let unicode: String = container.resolve("服务").await.unwrap();
    assert_eq!(unicode, "unicode_service");

    // Test with complex types
    let mut complex_data = HashMap::new();
    complex_data.insert("key1".to_string(), serde_json::json!({"nested": "value"}));
    complex_data.insert("key2".to_string(), serde_json::json!([1, 2, 3]));

    container.register("complex", complex_data.clone()).await;
    let resolved: HashMap<String, serde_json::Value> = container.resolve("complex").await.unwrap();
    assert_eq!(resolved, complex_data);

    // Test default container
    let default_container = Container::default();
    assert_eq!(default_container.list_services().await.len(), 0);
}

/// Test factory and singleton provider types directly
#[tokio::test]
async fn test_provider_types_directly() {
    // Test FactoryProvider directly
    let factory = FactoryProvider::new(|| "factory_value".to_string());
    let container = Container::new();

    let result = factory.provide(&container).await.unwrap();
    assert_eq!(result, "factory_value");

    assert!(factory.dependencies().is_empty());

    // Test SingletonProvider directly
    let inner_factory = FactoryProvider::new(|| {
        static COUNTER: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);
        COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    });

    let singleton = SingletonProvider::new(Box::new(inner_factory));

    // First call should create instance
    let val1 = singleton.provide(&container).await.unwrap();
    let val2 = singleton.provide(&container).await.unwrap();
    let val3 = singleton.provide(&container).await.unwrap();

    // All should be the same due to singleton behavior
    assert_eq!(val1, val2);
    assert_eq!(val2, val3);
}

/// Test ServiceInfo structure
#[tokio::test]
async fn test_service_info() {
    let info = ServiceInfo {
        name: "test_service".to_string(),
        service_type: "TestService".to_string(),
        dependencies: vec!["dep1".to_string(), "dep2".to_string()],
        is_singleton: true,
    };

    assert_eq!(info.name, "test_service");
    assert_eq!(info.service_type, "TestService");
    assert_eq!(info.dependencies.len(), 2);
    assert!(info.is_singleton);

    // Test cloning
    let cloned = info.clone();
    assert_eq!(cloned.name, info.name);
    assert_eq!(cloned.dependencies, info.dependencies);
    assert_eq!(cloned.is_singleton, info.is_singleton);

    // Test debug formatting
    let debug_str = format!("{info:?}");
    assert!(debug_str.contains("test_service"));
    assert!(debug_str.contains("TestService"));
}

/// Test concurrent access to container
#[tokio::test]
async fn test_container_concurrency() {
    let container = Arc::new(Container::new());

    // Register initial service
    container
        .register("shared", "shared_value".to_string())
        .await;

    let mut handles = vec![];

    // Spawn multiple tasks that access the container concurrently
    for i in 0..10 {
        let container_clone = Arc::clone(&container);
        let handle = tokio::spawn(async move {
            // Register a service specific to this task
            container_clone
                .register(&format!("service_{i}"), format!("value_{i}"))
                .await;

            // Resolve the shared service
            let shared: String = container_clone.resolve("shared").await.unwrap();
            assert_eq!(shared, "shared_value");

            // Resolve the task-specific service
            let specific: String = container_clone
                .resolve(&format!("service_{i}"))
                .await
                .unwrap();
            assert_eq!(specific, format!("value_{i}"));

            i
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    let mut results = vec![];
    for handle in handles {
        results.push(handle.await.unwrap());
    }
    for (i, result) in results.into_iter().enumerate() {
        assert_eq!(result, i);
    }

    // Verify all services were registered
    let services = container.list_services().await;
    assert!(services.len() >= 11); // shared + 10 task-specific

    // Verify we can resolve all services
    for i in 0..10 {
        let value: String = container.resolve(&format!("service_{i}")).await.unwrap();
        assert_eq!(value, format!("value_{i}"));
    }
}
