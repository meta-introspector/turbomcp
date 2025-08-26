use std::process::Command;

#[test]
fn test_server_binary_exists() {
    // Test that the server binary can be built
    let output = Command::new("cargo")
        .args(["build", "--bin", "turbomcp-server"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute cargo build");

    assert!(
        output.status.success(),
        "Server binary should build successfully. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_server_main_integration() {
    // Test that main.rs contains expected components
    let main_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/main.rs"),
    )
    .expect("Failed to read main.rs");

    // Verify main function exists
    assert!(main_content.contains("async fn main()"));

    // Verify it uses ServerBuilder
    assert!(main_content.contains("ServerBuilder::new()"));

    // Verify it uses default_config
    assert!(main_content.contains("default_config()"));

    // Verify it runs stdio transport
    assert!(main_content.contains("run_stdio()"));

    // Verify proper error handling
    assert!(main_content.contains("Result<(), Box<dyn std::error::Error>>"));
}

#[test]
fn test_main_dependencies() {
    // Test that main.rs imports are correct
    let main_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/main.rs"),
    )
    .expect("Failed to read main.rs");

    // Verify turbomcp_server imports
    assert!(main_content.contains("use turbomcp_server::{ServerBuilder, default_config}"));

    // Verify tokio main attribute
    assert!(main_content.contains("#[tokio::main]"));

    // Verify tracing initialization
    assert!(main_content.contains("tracing_subscriber::fmt::try_init()"));
}

#[test]
fn test_main_structure() {
    // Test the structure of main function
    let main_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/main.rs"),
    )
    .expect("Failed to read main.rs");

    // Count key components
    assert_eq!(main_content.matches("async fn main()").count(), 1);
    assert_eq!(main_content.matches("default_config()").count(), 1);
    assert_eq!(main_content.matches("ServerBuilder::new()").count(), 1);
    assert_eq!(main_content.matches("run_stdio()").count(), 1);
}

#[test]
fn test_main_error_handling() {
    // Test that main function has proper error handling
    let main_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/main.rs"),
    )
    .expect("Failed to read main.rs");

    // Verify error handling pattern
    assert!(main_content.contains("map_err(|e| e.into())"));

    // Verify return type includes error handling
    assert!(main_content.contains("Box<dyn std::error::Error>"));
}

// Test the configuration flow
#[tokio::test]
async fn test_config_creation() {
    use turbomcp_server::default_config;

    // Test that default config can be created
    let config = default_config();

    // Verify config has expected fields
    assert!(!config.name.is_empty());
    assert!(!config.version.is_empty());
}

#[tokio::test]
async fn test_server_builder_creation() {
    use turbomcp_server::{ServerBuilder, default_config};

    // Test server builder creation like main.rs
    let config = default_config();
    let server = ServerBuilder::new()
        .name(config.name.clone())
        .version(config.version.clone())
        .build();

    // Verify server was created successfully
    // Note: We can't easily test run_stdio() here as it would block
    drop(server); // Server creation completed successfully
}

// Test individual components that main.rs uses
#[test]
fn test_imports_compile() {
    // This test ensures all imports in main.rs are valid
    // If this test compiles, the imports are valid

    // Import what main.rs imports
    use turbomcp_server::{ServerBuilder, default_config};

    // Use them like main.rs does
    let _config = default_config();
    let _server = ServerBuilder::new();
}

#[test]
fn test_main_file_characteristics() {
    let main_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/main.rs");

    // Verify file exists
    assert!(main_path.exists(), "main.rs should exist");

    // Verify file is readable
    let main_content = std::fs::read_to_string(&main_path).expect("Should be able to read main.rs");

    // Verify it's not empty
    assert!(
        !main_content.trim().is_empty(),
        "main.rs should not be empty"
    );

    // Verify reasonable size (not too small or too large)
    assert!(
        main_content.len() > 100,
        "main.rs should have substantial content"
    );
    assert!(
        main_content.len() < 5000,
        "main.rs should be reasonably sized"
    );
}

#[test]
fn test_tokio_main_attribute() {
    let main_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/main.rs"),
    )
    .expect("Failed to read main.rs");

    // Verify tokio main attribute is applied correctly
    let lines: Vec<&str> = main_content.lines().collect();
    let mut found_attribute = false;
    let mut found_main = false;

    for (i, line) in lines.iter().enumerate() {
        if line.contains("#[tokio::main]") {
            found_attribute = true;
            // Next non-empty line should contain main function
            for next_line in lines.iter().skip(i + 1) {
                if !next_line.trim().is_empty() {
                    if next_line.contains("async fn main()") {
                        found_main = true;
                    }
                    break;
                }
            }
            break;
        }
    }

    assert!(found_attribute, "Should have #[tokio::main] attribute");
    assert!(
        found_main,
        "Should have async main function after attribute"
    );
}
