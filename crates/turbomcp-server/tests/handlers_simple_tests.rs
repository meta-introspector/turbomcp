//! Simple tests for handler utilities and basic functionality

use turbomcp_protocol::types::*;
use turbomcp_server::handlers::*;

#[test]
fn test_handler_wrapper_new() {
    let tool_def = Tool {
        name: "test_tool".to_string(),
        title: Some("Test Tool".to_string()),
        description: Some("A test tool".to_string()),
        input_schema: ToolInputSchema {
            schema_type: "object".to_string(),
            properties: None,
            required: None,
            additional_properties: None,
        },
        output_schema: None,
        annotations: None,
        meta: None,
    };

    let handler = FunctionToolHandler::new(tool_def, |_request, _ctx| async move {
        use turbomcp_server::ServerError;
        Err(ServerError::handler("test"))
    });

    let wrapper = HandlerWrapper::new(handler);
    assert_eq!(wrapper.metadata().name, "unnamed");
    assert_eq!(wrapper.metadata().version, "1.0.0");
}

#[test]
fn test_handler_metadata_default() {
    let metadata = HandlerMetadata::default();
    assert_eq!(metadata.name, "unnamed");
    assert_eq!(metadata.version, "1.0.0");
    assert!(metadata.metrics_enabled);
    assert!(metadata.rate_limit.is_none());
    assert!(metadata.created_at.timestamp() > 0);
}

#[test]
fn test_handler_metadata_with_values() {
    use std::collections::HashMap;

    let mut config = HashMap::new();
    config.insert("timeout".to_string(), serde_json::Value::Number(30.into()));

    let metadata = HandlerMetadata {
        name: "test_handler".to_string(),
        version: "2.0.0".to_string(),
        description: Some("Test handler".to_string()),
        tags: vec!["test".to_string(), "handler".to_string()],
        created_at: chrono::Utc::now(),
        config,
        metrics_enabled: false,
        rate_limit: Some(100),
        allowed_roles: Some(vec!["admin".to_string()]),
    };

    assert_eq!(metadata.name, "test_handler");
    assert_eq!(metadata.version, "2.0.0");
    assert_eq!(metadata.tags.len(), 2);
    assert!(!metadata.metrics_enabled);
    assert_eq!(metadata.rate_limit, Some(100));
    assert_eq!(metadata.config.len(), 1);
}

#[test]
fn test_function_tool_handler_definition() {
    let tool = Tool {
        name: "echo".to_string(),
        title: Some("Echo Tool".to_string()),
        description: Some("Echoes input".to_string()),
        input_schema: ToolInputSchema {
            schema_type: "object".to_string(),
            properties: None,
            required: None,
            additional_properties: None,
        },
        output_schema: None,
        annotations: None,
        meta: None,
    };

    let handler = FunctionToolHandler::new(tool.clone(), |_request, _ctx| async move {
        use turbomcp_server::ServerError;
        Err(ServerError::handler("test"))
    });

    let definition = handler.tool_definition();
    assert_eq!(definition.name, tool.name);
    assert_eq!(definition.title, tool.title);
    assert_eq!(definition.description, tool.description);
}

#[test]
fn test_function_tool_handler_roles() {
    let tool = Tool {
        name: "admin_tool".to_string(),
        title: Some("Admin Tool".to_string()),
        description: Some("Admin only".to_string()),
        input_schema: ToolInputSchema {
            schema_type: "object".to_string(),
            properties: None,
            required: None,
            additional_properties: None,
        },
        output_schema: None,
        annotations: None,
        meta: None,
    };

    let roles = vec!["admin".to_string(), "superuser".to_string()];
    let handler = FunctionToolHandler::new_with_roles(
        tool,
        |_request, _ctx| async move {
            use turbomcp_server::ServerError;
            Err(ServerError::handler("test"))
        },
        Some(roles.clone()),
    );

    let allowed_roles = handler.allowed_roles();
    assert!(allowed_roles.is_some());
    assert_eq!(allowed_roles.unwrap(), roles.as_slice());
}

#[test]
fn test_function_tool_handler_no_roles() {
    let tool = Tool {
        name: "public_tool".to_string(),
        title: Some("Public Tool".to_string()),
        description: Some("Available to all".to_string()),
        input_schema: ToolInputSchema {
            schema_type: "object".to_string(),
            properties: None,
            required: None,
            additional_properties: None,
        },
        output_schema: None,
        annotations: None,
        meta: None,
    };

    let handler = FunctionToolHandler::new(tool, |_request, _ctx| async move {
        use turbomcp_server::ServerError;
        Err(ServerError::handler("test"))
    });

    let allowed_roles = handler.allowed_roles();
    assert!(allowed_roles.is_none());
}

#[test]
fn test_function_prompt_handler_definition() {
    let prompt = Prompt {
        name: "greeting".to_string(),
        title: Some("Greeting Prompt".to_string()),
        description: Some("Generates greetings".to_string()),
        arguments: None,
        meta: None,
    };

    let handler = FunctionPromptHandler::new(prompt.clone(), |_request, _ctx| async move {
        use turbomcp_server::ServerError;
        Err(ServerError::handler("test"))
    });

    let definition = handler.prompt_definition();
    assert_eq!(definition.name, prompt.name);
    assert_eq!(definition.title, prompt.title);
    assert_eq!(definition.description, prompt.description);
}

#[test]
fn test_function_resource_handler_definition() {
    let resource = Resource {
        name: "test_file".to_string(),
        title: Some("Test File".to_string()),
        uri: "file://test.txt".to_string(),
        description: Some("Test resource".to_string()),
        mime_type: Some("text/plain".to_string()),
        annotations: None,
        size: None,
        meta: None,
    };

    let handler = FunctionResourceHandler::new(resource.clone(), |_request, _ctx| async move {
        use turbomcp_server::ServerError;
        Err(ServerError::not_found("test"))
    });

    let definition = handler.resource_definition();
    assert_eq!(definition.name, resource.name);
    assert_eq!(definition.uri, resource.uri);
    assert_eq!(definition.mime_type, resource.mime_type);
}

#[tokio::test]
async fn test_function_resource_handler_exists() {
    let resource = Resource {
        name: "test_file".to_string(),
        title: Some("Test File".to_string()),
        uri: "file://test.txt".to_string(),
        description: None,
        mime_type: None,
        annotations: None,
        size: None,
        meta: None,
    };

    let handler = FunctionResourceHandler::new(resource, |_request, _ctx| async move {
        use turbomcp_server::ServerError;
        Err(ServerError::not_found("test"))
    });

    // Default exists function always returns true
    let exists = handler.exists("file://test.txt").await;
    assert!(exists);
}

// Test the utility functions
#[test]
fn test_utils_tool() {
    use turbomcp_server::handlers::utils::tool;

    let handler = tool("echo", "Echo tool", |_request, _ctx| async move {
        use turbomcp_server::ServerError;
        Err(ServerError::handler("test"))
    });

    let definition = handler.tool_definition();
    assert_eq!(definition.name, "echo");
    assert_eq!(definition.title, Some("echo".to_string()));
    assert_eq!(definition.description, Some("Echo tool".to_string()));
    assert_eq!(definition.input_schema.schema_type, "object");
}

#[test]
fn test_utils_prompt() {
    use turbomcp_server::handlers::utils::prompt;

    let handler = prompt("greeting", "Greeting prompt", |_request, _ctx| async move {
        use turbomcp_server::ServerError;
        Err(ServerError::handler("test"))
    });

    let definition = handler.prompt_definition();
    assert_eq!(definition.name, "greeting");
    assert_eq!(definition.title, Some("greeting".to_string()));
    assert_eq!(definition.description, Some("Greeting prompt".to_string()));
}

#[test]
fn test_utils_resource() {
    use turbomcp_server::handlers::utils::resource;

    let handler = resource("file://test", "Test File", |_request, _ctx| async move {
        use turbomcp_server::ServerError;
        Err(ServerError::not_found("test"))
    });

    let definition = handler.resource_definition();
    assert_eq!(definition.name, "Test File");
    assert_eq!(definition.uri, "file://test");
    assert_eq!(definition.title, Some("Test File".to_string()));
}

// Test debug formatting
#[test]
fn test_function_tool_handler_debug() {
    let tool = Tool {
        name: "debug_tool".to_string(),
        title: Some("Debug Tool".to_string()),
        description: Some("For testing debug output".to_string()),
        input_schema: ToolInputSchema {
            schema_type: "object".to_string(),
            properties: None,
            required: None,
            additional_properties: None,
        },
        output_schema: None,
        annotations: None,
        meta: None,
    };

    let handler = FunctionToolHandler::new(tool, |_request, _ctx| async move {
        use turbomcp_server::ServerError;
        Err(ServerError::handler("test"))
    });

    let debug_str = format!("{handler:?}");
    assert!(debug_str.contains("FunctionToolHandler"));
    assert!(debug_str.contains("debug_tool"));
}

#[test]
fn test_function_prompt_handler_debug() {
    let prompt = Prompt {
        name: "debug_prompt".to_string(),
        title: Some("Debug Prompt".to_string()),
        description: Some("For testing debug output".to_string()),
        arguments: None,
        meta: None,
    };

    let handler = FunctionPromptHandler::new(prompt, |_request, _ctx| async move {
        use turbomcp_server::ServerError;
        Err(ServerError::handler("test"))
    });

    let debug_str = format!("{handler:?}");
    assert!(debug_str.contains("FunctionPromptHandler"));
    assert!(debug_str.contains("debug_prompt"));
}

#[test]
fn test_function_resource_handler_debug() {
    let resource = Resource {
        name: "debug_resource".to_string(),
        title: Some("Debug Resource".to_string()),
        uri: "file://debug.txt".to_string(),
        description: None,
        mime_type: None,
        annotations: None,
        size: None,
        meta: None,
    };

    let handler = FunctionResourceHandler::new(resource, |_request, _ctx| async move {
        use turbomcp_server::ServerError;
        Err(ServerError::not_found("test"))
    });

    let debug_str = format!("{handler:?}");
    assert!(debug_str.contains("FunctionResourceHandler"));
    assert!(debug_str.contains("debug_resource"));
}

#[test]
fn test_handler_wrapper_debug() {
    let tool = Tool {
        name: "wrapper_test".to_string(),
        title: Some("Wrapper Test".to_string()),
        description: Some("For testing wrapper debug".to_string()),
        input_schema: ToolInputSchema {
            schema_type: "object".to_string(),
            properties: None,
            required: None,
            additional_properties: None,
        },
        output_schema: None,
        annotations: None,
        meta: None,
    };

    let handler = FunctionToolHandler::new(tool, |_request, _ctx| async move {
        use turbomcp_server::ServerError;
        Err(ServerError::handler("test"))
    });

    let wrapper = HandlerWrapper::new(handler);
    let debug_str = format!("{wrapper:?}");
    assert!(debug_str.contains("HandlerWrapper"));
    assert!(debug_str.contains("metadata"));
}

#[test]
fn test_handler_wrapper_with_metadata() {
    use std::collections::HashMap;

    let tool = Tool {
        name: "meta_test".to_string(),
        title: Some("Metadata Test".to_string()),
        description: Some("For testing metadata".to_string()),
        input_schema: ToolInputSchema {
            schema_type: "object".to_string(),
            properties: None,
            required: None,
            additional_properties: None,
        },
        output_schema: None,
        annotations: None,
        meta: None,
    };

    let handler = FunctionToolHandler::new(tool, |_request, _ctx| async move {
        use turbomcp_server::ServerError;
        Err(ServerError::handler("test"))
    });

    let mut config = HashMap::new();
    config.insert(
        "test_key".to_string(),
        serde_json::Value::String("test_value".to_string()),
    );

    let metadata = HandlerMetadata {
        name: "custom_handler".to_string(),
        version: "1.5.0".to_string(),
        description: Some("Custom handler for testing".to_string()),
        tags: vec!["test".to_string(), "custom".to_string()],
        created_at: chrono::Utc::now(),
        config,
        metrics_enabled: true,
        rate_limit: Some(50),
        allowed_roles: Some(vec!["test_role".to_string()]),
    };

    let wrapper = HandlerWrapper::with_metadata(handler, metadata);

    assert_eq!(wrapper.metadata().name, "custom_handler");
    assert_eq!(wrapper.metadata().version, "1.5.0");
    assert_eq!(wrapper.metadata().tags.len(), 2);
    assert_eq!(wrapper.metadata().rate_limit, Some(50));
    assert_eq!(wrapper.metadata().config.len(), 1);
}

#[test]
fn test_handler_wrapper_update_metadata() {
    let tool = Tool {
        name: "update_test".to_string(),
        title: Some("Update Test".to_string()),
        description: Some("For testing metadata updates".to_string()),
        input_schema: ToolInputSchema {
            schema_type: "object".to_string(),
            properties: None,
            required: None,
            additional_properties: None,
        },
        output_schema: None,
        annotations: None,
        meta: None,
    };

    let handler = FunctionToolHandler::new(tool, |_request, _ctx| async move {
        use turbomcp_server::ServerError;
        Err(ServerError::handler("test"))
    });

    let mut wrapper = HandlerWrapper::new(handler);

    wrapper.update_metadata(|meta| {
        meta.name = "updated_handler".to_string();
        meta.version = "2.0.0".to_string();
        meta.metrics_enabled = false;
        meta.rate_limit = Some(200);
    });

    assert_eq!(wrapper.metadata().name, "updated_handler");
    assert_eq!(wrapper.metadata().version, "2.0.0");
    assert!(!wrapper.metadata().metrics_enabled);
    assert_eq!(wrapper.metadata().rate_limit, Some(200));
}
