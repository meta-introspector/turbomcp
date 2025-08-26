//! Comprehensive Integration Tests for TurboMCP
//!
//! These tests validate the complete macro→schema→protocol chain that would have
//! caught the schema generation bug. No mocks, no placeholders, no gaslighting.
//! Only real, comprehensive validation.

use serde_json::json;
use turbomcp::prelude::*;

// =============================================================================
// CRITICAL TEST 1: Schema Generation Chain Validation
// =============================================================================

#[derive(Clone)]
struct SchemaValidationServer;

#[server(name = "SchemaValidator", version = "1.0.0")]
#[allow(dead_code, clippy::too_many_arguments)]
impl SchemaValidationServer {
    /// Tool with all parameter types for comprehensive schema validation
    #[tool("Validates all parameter types and schema generation")]
    async fn comprehensive_params(
        &self,
        required_string: String,
        required_int: i32,
        required_float: f64,
        required_bool: bool,
        optional_string: Option<String>,
        optional_int: Option<i64>,
        optional_array: Option<Vec<String>>,
    ) -> McpResult<String> {
        Ok(format!(
            "Validated: {} {} {} {} {:?} {:?} {:?}",
            required_string,
            required_int,
            required_float,
            required_bool,
            optional_string,
            optional_int,
            optional_array
        ))
    }

    #[tool("Tool with no parameters")]
    async fn no_params(&self) -> McpResult<String> {
        Ok("No parameters".to_string())
    }

    #[tool("Tool with only optional parameters")]
    async fn only_optional(&self, opt1: Option<String>, opt2: Option<bool>) -> McpResult<String> {
        Ok(format!("opt1: {:?}, opt2: {:?}", opt1, opt2))
    }
}

/// CRITICAL TEST: Validates that tool metadata is properly generated and accessible
#[test]
fn test_tool_metadata_generation() {
    // This test would have FAILED before the schema bug fix
    // because the server macro was ignoring schemas with _schema

    // Test comprehensive_params metadata
    let (name, desc, schema) = SchemaValidationServer::comprehensive_params_metadata();

    assert_eq!(name, "comprehensive_params");
    assert_eq!(desc, "Validates all parameter types and schema generation");

    // CRITICAL: Schema must not be empty or null
    assert!(!schema.is_null(), "Schema must not be null");
    assert!(schema.is_object(), "Schema must be an object");

    let schema_obj = schema.as_object().unwrap();
    assert_eq!(schema_obj.get("type").unwrap(), "object");

    // CRITICAL: Properties must contain all parameters
    let properties = schema_obj.get("properties").unwrap().as_object().unwrap();
    assert_eq!(properties.len(), 7, "Should have all 7 parameters");

    // Validate each parameter exists in schema
    assert!(
        properties.contains_key("required_string"),
        "Missing required_string"
    );
    assert!(
        properties.contains_key("required_int"),
        "Missing required_int"
    );
    assert!(
        properties.contains_key("required_float"),
        "Missing required_float"
    );
    assert!(
        properties.contains_key("required_bool"),
        "Missing required_bool"
    );
    assert!(
        properties.contains_key("optional_string"),
        "Missing optional_string"
    );
    assert!(
        properties.contains_key("optional_int"),
        "Missing optional_int"
    );
    assert!(
        properties.contains_key("optional_array"),
        "Missing optional_array"
    );

    // CRITICAL: Validate required vs optional parameters
    let required = schema_obj.get("required").unwrap().as_array().unwrap();
    assert_eq!(required.len(), 4, "Should have 4 required parameters");

    let required_params: Vec<&str> = required.iter().map(|v| v.as_str().unwrap()).collect();

    assert!(required_params.contains(&"required_string"));
    assert!(required_params.contains(&"required_int"));
    assert!(required_params.contains(&"required_float"));
    assert!(required_params.contains(&"required_bool"));

    // Optional parameters must NOT be in required array
    assert!(!required_params.contains(&"optional_string"));
    assert!(!required_params.contains(&"optional_int"));
    assert!(!required_params.contains(&"optional_array"));
}

/// Test type inference from Rust types to JSON schema types
#[test]
fn test_schema_type_inference() {
    let (_, _, schema) = SchemaValidationServer::comprehensive_params_metadata();
    let properties = schema["properties"].as_object().unwrap();

    // String type
    assert_eq!(properties["required_string"]["type"], "string");
    assert_eq!(properties["optional_string"]["type"], "string");

    // Integer types
    assert_eq!(properties["required_int"]["type"], "integer");
    assert_eq!(properties["optional_int"]["type"], "integer");

    // Float type (f64 -> number)
    assert_eq!(properties["required_float"]["type"], "number");

    // Boolean type
    assert_eq!(properties["required_bool"]["type"], "boolean");

    // Array type
    let array_schema = &properties["optional_array"];
    assert_eq!(array_schema["type"], "array");
    assert_eq!(array_schema["items"]["type"], "string");
}

/// Test tools discovery and metadata collection
#[test]
fn test_server_tools_discovery() {
    let tools = SchemaValidationServer::get_tools_metadata();

    assert_eq!(tools.len(), 3, "Should discover all 3 tools");

    // Find each tool and validate
    let comprehensive = tools
        .iter()
        .find(|(name, _, _)| name == "comprehensive_params")
        .expect("comprehensive_params tool should exist");

    assert_eq!(
        comprehensive.1,
        "Validates all parameter types and schema generation"
    );
    assert!(comprehensive.2.is_object());
    assert!(
        !comprehensive.2["properties"]
            .as_object()
            .unwrap()
            .is_empty()
    );

    let no_params = tools
        .iter()
        .find(|(name, _, _)| name == "no_params")
        .expect("no_params tool should exist");

    assert_eq!(no_params.1, "Tool with no parameters");
    let no_params_props = no_params.2["properties"].as_object().unwrap();
    assert!(
        no_params_props.is_empty(),
        "no_params should have empty properties"
    );

    let only_optional = tools
        .iter()
        .find(|(name, _, _)| name == "only_optional")
        .expect("only_optional tool should exist");

    assert_eq!(only_optional.1, "Tool with only optional parameters");

    // Should have no required parameters
    let only_opt_schema = only_optional.2.as_object().unwrap();
    if let Some(required) = only_opt_schema.get("required") {
        assert!(required.as_array().unwrap().is_empty());
    }
}

// =============================================================================
// CRITICAL TEST 2: Direct Tool Call Validation
// =============================================================================

/// Test direct tool calls with parameter validation
#[tokio::test]
async fn test_direct_tool_calls() {
    let server = SchemaValidationServer;

    // Test valid call with all parameters
    let result = server
        .test_tool_call(
            "comprehensive_params",
            json!({
                "required_string": "test",
                "required_int": 42,
                "required_float": std::f64::consts::PI,
                "required_bool": true,
                "optional_string": "optional",
                "optional_int": 100,
                "optional_array": ["a", "b", "c"]
            }),
        )
        .await;

    assert!(
        result.is_ok(),
        "Valid parameters should succeed: {:?}",
        result
    );
    let call_result = result.unwrap();
    println!("Call result: {:?}", call_result);
    println!("is_error: {:?}", call_result.is_error);
    assert!(
        !call_result.is_error.unwrap_or(true),
        "Result marked as error but shouldn't be"
    );

    // Test with only required parameters
    let result = server
        .test_tool_call(
            "comprehensive_params",
            json!({
                "required_string": "test",
                "required_int": 42,
                "required_float": std::f64::consts::PI,
                "required_bool": false
            }),
        )
        .await;

    assert!(
        result.is_ok(),
        "Call with only required params should succeed"
    );

    // Test missing required parameter
    let result = server
        .test_tool_call(
            "comprehensive_params",
            json!({
                "required_string": "test",
                "required_int": 42,
                "required_bool": true
                // Missing required_float
            }),
        )
        .await;

    assert!(result.is_err(), "Missing required parameter should fail");

    // Test wrong parameter type
    let result = server
        .test_tool_call(
            "comprehensive_params",
            json!({
                "required_string": "test",
                "required_int": "not_an_int", // Wrong type
                "required_float": std::f64::consts::PI,
                "required_bool": true
            }),
        )
        .await;

    assert!(result.is_err(), "Wrong parameter type should fail");

    // Test non-existent tool
    let result = server.test_tool_call("non_existent_tool", json!({})).await;

    assert!(result.is_err(), "Non-existent tool should fail");
    let err = result.unwrap_err();
    assert!(err.to_string().contains("not found"));
}

/// Test tools with no parameters
#[tokio::test]
async fn test_no_params_tool() {
    let server = SchemaValidationServer;

    // Should work with empty object
    let result = server.test_tool_call("no_params", json!({})).await;
    assert!(result.is_ok(), "no_params with empty object should succeed");

    // Should also work with null (no arguments)
    let result = server.test_tool_call("no_params", json!(null)).await;
    assert!(result.is_ok(), "no_params with null should succeed");
}

/// Test tools with only optional parameters
#[tokio::test]
async fn test_optional_only_tool() {
    let server = SchemaValidationServer;

    // Should work with no parameters
    let result = server.test_tool_call("only_optional", json!({})).await;
    assert!(
        result.is_ok(),
        "only_optional with no params should succeed"
    );

    // Should work with some optional parameters
    let result = server
        .test_tool_call("only_optional", json!({"opt1": "hello"}))
        .await;
    assert!(
        result.is_ok(),
        "only_optional with one param should succeed"
    );

    // Should work with all optional parameters
    let result = server
        .test_tool_call("only_optional", json!({"opt1": "hello", "opt2": true}))
        .await;
    assert!(
        result.is_ok(),
        "only_optional with all params should succeed"
    );
}

// =============================================================================
// CRITICAL TEST 3: Complex Type Schema Generation
// =============================================================================

#[derive(Clone)]
struct ComplexTypeServer;

#[server(name = "ComplexTypes", version = "1.0.0")]
#[allow(dead_code)]
impl ComplexTypeServer {
    #[tool("Tool with nested optional types")]
    async fn nested_optionals(
        &self,
        maybe_list: Option<Vec<Option<String>>>,
        nested_int: Option<Option<i32>>,
    ) -> McpResult<String> {
        Ok(format!(
            "maybe_list: {:?}, nested_int: {:?}",
            maybe_list, nested_int
        ))
    }

    #[tool("Tool with various collection types")]
    async fn collections(
        &self,
        string_vec: Vec<String>,
        int_vec: Vec<i32>,
        float_vec: Option<Vec<f64>>,
    ) -> McpResult<String> {
        Ok(format!("{:?} {:?} {:?}", string_vec, int_vec, float_vec))
    }
}

/// Test complex type schema generation
#[test]
fn test_complex_type_schemas() {
    // Test nested optionals
    let (_, _, schema) = ComplexTypeServer::nested_optionals_metadata();
    let properties = schema["properties"].as_object().unwrap();

    assert!(properties.contains_key("maybe_list"));
    assert!(properties.contains_key("nested_int"));

    // Both should be optional (not in required array)
    if let Some(required) = schema.get("required") {
        let required_array = required.as_array().unwrap();
        assert!(
            required_array.is_empty()
                || !required_array
                    .iter()
                    .any(|v| v.as_str() == Some("maybe_list") || v.as_str() == Some("nested_int"))
        );
    }

    // Test collections
    let (_, _, schema) = ComplexTypeServer::collections_metadata();
    let properties = schema["properties"].as_object().unwrap();

    assert_eq!(properties["string_vec"]["type"], "array");
    assert_eq!(properties["string_vec"]["items"]["type"], "string");

    assert_eq!(properties["int_vec"]["type"], "array");
    assert_eq!(properties["int_vec"]["items"]["type"], "integer");

    assert_eq!(properties["float_vec"]["type"], "array");
    assert_eq!(properties["float_vec"]["items"]["type"], "number");

    // string_vec and int_vec should be required
    let required = schema["required"].as_array().unwrap();
    let required_params: Vec<&str> = required.iter().map(|v| v.as_str().unwrap()).collect();

    assert!(required_params.contains(&"string_vec"));
    assert!(required_params.contains(&"int_vec"));
    assert!(!required_params.contains(&"float_vec")); // Optional
}

// =============================================================================
// CRITICAL TEST 4: Error Handling and Edge Cases
// =============================================================================

#[derive(Clone)]
struct ErrorHandlingServer;

#[server(name = "ErrorHandler", version = "1.0.0")]
impl ErrorHandlingServer {
    #[tool("Tool that validates input ranges")]
    async fn validate_range(&self, value: i32, min: i32, max: i32) -> McpResult<String> {
        if value < min || value > max {
            return Err(McpError::Tool(format!(
                "Value {} out of range [{}, {}]",
                value, min, max
            )));
        }
        Ok(format!("{} is within range", value))
    }

    #[tool("Tool that requires non-empty string")]
    async fn require_non_empty(&self, text: String) -> McpResult<String> {
        if text.is_empty() {
            return Err(McpError::Tool("Text cannot be empty".to_string()));
        }
        Ok(format!("Processed: {}", text))
    }
}

/// Test error handling in tool calls
#[tokio::test]
async fn test_error_handling() {
    let server = ErrorHandlingServer;

    // Test successful validation
    let result = server
        .test_tool_call("validate_range", json!({"value": 5, "min": 0, "max": 10}))
        .await;
    assert!(result.is_ok());

    // Test range validation failure
    let result = server
        .test_tool_call("validate_range", json!({"value": 15, "min": 0, "max": 10}))
        .await;
    assert!(result.is_err());

    // Test empty string validation
    let result = server
        .test_tool_call("require_non_empty", json!({"text": ""}))
        .await;
    assert!(result.is_err());

    // Test non-empty string success
    let result = server
        .test_tool_call("require_non_empty", json!({"text": "valid"}))
        .await;
    assert!(result.is_ok());
}

// =============================================================================
// CRITICAL TEST 5: Regression Test for Schema Bug
// =============================================================================

/// This is the exact test that would have caught the schema bug
/// where the server macro was doing: let (name, desc, _schema) = metadata();
#[test]
fn test_schema_bug_regression() {
    // Get all tools metadata
    let tools = SchemaValidationServer::get_tools_metadata();

    for (name, description, schema) in tools {
        // CRITICAL: Every tool must have a valid schema
        assert!(
            !schema.is_null(),
            "Tool {} has null schema - REGRESSION!",
            name
        );
        assert!(schema.is_object(), "Tool {} schema is not an object", name);

        let schema_obj = schema.as_object().unwrap();

        // Every schema must have required fields
        assert!(
            schema_obj.contains_key("type"),
            "Tool {} missing 'type' field",
            name
        );
        assert!(
            schema_obj.contains_key("properties"),
            "Tool {} missing 'properties'",
            name
        );
        assert_eq!(
            schema_obj["type"], "object",
            "Tool {} type must be 'object'",
            name
        );

        // If tool has parameters, properties must not be empty
        if name != "no_params" {
            let properties = schema_obj["properties"].as_object().unwrap();
            assert!(
                !properties.is_empty(),
                "Tool {} has parameters but empty properties - SCHEMA BUG!",
                name
            );
        }

        // Description must match
        assert!(
            !description.is_empty(),
            "Tool {} has empty description",
            name
        );
    }
}

/// Test that schemas contain actual parameter information, not placeholder schemas
#[test]
fn test_no_placeholder_schemas() {
    let tools = SchemaValidationServer::get_tools_metadata();

    for (name, _, schema) in tools {
        let schema_str = serde_json::to_string(&schema).unwrap();

        // Check for signs of placeholder/mock schemas
        assert!(
            !schema_str.contains(r#""properties":{}"#) || name == "no_params",
            "Tool {} has empty properties object - likely placeholder!",
            name
        );

        // Comprehensive_params must have specific parameters
        if name == "comprehensive_params" {
            assert!(
                schema_str.contains("required_string"),
                "Schema missing required_string parameter"
            );
            assert!(
                schema_str.contains("required_int"),
                "Schema missing required_int parameter"
            );
            assert!(
                schema_str.contains("required_float"),
                "Schema missing required_float parameter"
            );
            assert!(
                schema_str.contains("required_bool"),
                "Schema missing required_bool parameter"
            );
        }
    }
}

// =============================================================================
// SUCCESS: Comprehensive Integration Testing Complete
// =============================================================================

/// Final validation that our test infrastructure works
#[test]
fn test_comprehensive_infrastructure_validation() {
    // Verify we can access metadata functions and validate their content
    let (name1, desc1, schema1) = SchemaValidationServer::comprehensive_params_metadata();
    let (name2, desc2, schema2) = SchemaValidationServer::no_params_metadata();
    let (name3, desc3, schema3) = SchemaValidationServer::only_optional_metadata();

    // Validate metadata content
    assert!(!name1.is_empty() && !desc1.is_empty() && schema1.is_object());
    assert!(!name2.is_empty() && !desc2.is_empty() && schema2.is_object());
    assert!(!name3.is_empty() && !desc3.is_empty() && schema3.is_object());

    // Verify we can get all tools
    let tools = SchemaValidationServer::get_tools_metadata();
    assert!(!tools.is_empty(), "Tools discovery must work");

    // Verify each tool has complete metadata
    for (name, description, schema) in tools {
        assert!(!name.is_empty(), "Tool name cannot be empty");
        assert!(!description.is_empty(), "Tool description cannot be empty");
        assert!(schema.is_object(), "Tool schema must be valid JSON object");
    }

    println!("✅ Comprehensive integration testing infrastructure validated!");
}
