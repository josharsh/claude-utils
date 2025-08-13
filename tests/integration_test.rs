use claude_utils::{
    clipboard::{ClipboardContent, ClipboardManager},
    file_manager::{FileManager, FileManagerConfig},
};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_clipboard_text_roundtrip() {
    let clipboard = ClipboardManager::new().expect("Failed to create clipboard manager");

    // Set text
    let test_text = "Hello, Claude-Utils!";
    let content = ClipboardContent::Text {
        data: test_text.to_string(),
        truncated: None,
    };

    clipboard
        .set_content(&content)
        .expect("Failed to set clipboard");

    // Small delay to ensure clipboard is updated
    sleep(Duration::from_millis(100)).await;

    // Get text
    let retrieved = clipboard.get_content().expect("Failed to get clipboard");

    match retrieved.content {
        ClipboardContent::Text { data, .. } => {
            assert_eq!(data, test_text);
        }
        _ => panic!("Expected text content"),
    }
}

#[tokio::test]
async fn test_file_staging() {
    let file_manager = FileManager::new(FileManagerConfig::default())
        .await
        .expect("Failed to create file manager");

    // Stage some test data
    let test_data = b"Test image data";
    let staged = file_manager
        .stage_image(test_data, "png")
        .await
        .expect("Failed to stage file");

    // Verify file exists
    assert!(staged.path.exists());
    assert_eq!(staged.size, test_data.len());
    assert_eq!(staged.format, "png");

    // Verify deduplication
    let staged2 = file_manager
        .stage_image(test_data, "png")
        .await
        .expect("Failed to stage file again");

    assert_eq!(staged.path, staged2.path);
}

#[test]
fn test_mcp_protocol_serialization() {
    use claude_utils::mcp::protocol::*;

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(1)),
        method: "initialize".to_string(),
        params: None,
    };

    let json = serde_json::to_string(&request).expect("Failed to serialize");
    assert!(json.contains("\"jsonrpc\":\"2.0\""));
    assert!(json.contains("\"method\":\"initialize\""));

    let response = create_success_response(
        Some(serde_json::json!(1)),
        serde_json::json!({"status": "ok"}),
    );
    let response_json = serde_json::to_string(&response).expect("Failed to serialize response");
    assert!(response_json.contains("\"result\""));
    assert!(!response_json.contains("\"error\""));
}
