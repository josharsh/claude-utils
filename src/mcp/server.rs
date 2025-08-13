use axum::response::sse::{Event, KeepAlive, Sse};
use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tracing::{error, info};

use crate::{
    clipboard::{ClipboardContent, ClipboardManager},
    file_manager::FileManager,
    mcp::{auth::AuthManager, protocol::*},
    ClaudeUtilsError, Result,
};

#[derive(Clone)]
pub struct McpServerState {
    pub clipboard: Arc<ClipboardManager>,
    pub file_manager: Arc<FileManager>,
    pub auth_manager: Arc<AuthManager>,
    pub initialized: Arc<RwLock<bool>>,
}

#[derive(Debug, Deserialize)]
pub struct AuthQuery {
    token: Option<String>,
}

pub struct McpServer {
    state: McpServerState,
    port: u16,
    host: String,
}

impl McpServer {
    pub async fn new(
        clipboard: Arc<ClipboardManager>,
        file_manager: Arc<FileManager>,
        auth_manager: AuthManager,
        port: u16,
        host: String,
    ) -> Result<Self> {
        let state = McpServerState {
            clipboard,
            file_manager,
            auth_manager: Arc::new(auth_manager),
            initialized: Arc::new(RwLock::new(false)),
        };

        Ok(Self { state, port, host })
    }

    pub async fn run(self) -> Result<()> {
        let app = Router::new()
            .route("/health", get(health_handler))
            .route("/", post(jsonrpc_handler))
            .route("/rpc", post(jsonrpc_handler))
            .route("/sse", get(sse_handler))
            .layer(CorsLayer::permissive())
            .with_state(self.state);

        let addr = format!("{}:{}", self.host, self.port);
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(|e| ClaudeUtilsError::Server(format!("Failed to bind to {addr}: {e}")))?;

        info!("MCP server listening on http://{}", addr);

        axum::serve(listener, app)
            .await
            .map_err(|e| ClaudeUtilsError::Server(e.to_string()))?;

        Ok(())
    }
}

// Health check endpoint
async fn health_handler(State(state): State<McpServerState>) -> impl IntoResponse {
    let token = state.auth_manager.get_token().await;

    Json(json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
        "platform": std::env::consts::OS,
        "capabilities": ["text", "image", "watch"],
        "auth_required": token.is_some(),
    }))
}

// Main JSON-RPC handler
async fn jsonrpc_handler(
    State(state): State<McpServerState>,
    headers: HeaderMap,
    Json(request): Json<Value>,
) -> Response {
    // Check authentication
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    if !state.auth_manager.validate_token(auth_header).await {
        return (
            StatusCode::UNAUTHORIZED,
            Json(create_error_response(
                None,
                -32000,
                "Authentication required".to_string(),
            )),
        )
            .into_response();
    }

    // Handle batch requests
    if request.is_array() {
        let requests = request.as_array().unwrap();
        let mut responses = Vec::new();

        for req in requests {
            if let Ok(rpc_req) = serde_json::from_value::<JsonRpcRequest>(req.clone()) {
                responses.push(handle_single_request(state.clone(), rpc_req).await);
            }
        }

        return Json(Value::Array(
            responses
                .into_iter()
                .map(|r| serde_json::to_value(r).unwrap())
                .collect(),
        ))
        .into_response();
    }

    // Handle single request
    match serde_json::from_value::<JsonRpcRequest>(request) {
        Ok(rpc_req) => {
            let response = handle_single_request(state, rpc_req).await;
            Json(response).into_response()
        }
        Err(_) => Json(create_error_response(
            None,
            PARSE_ERROR,
            "Invalid JSON-RPC request".to_string(),
        ))
        .into_response(),
    }
}

async fn handle_single_request(state: McpServerState, request: JsonRpcRequest) -> JsonRpcResponse {
    match request.method.as_str() {
        INITIALIZE => handle_initialize(state, request).await,
        INITIALIZED => handle_initialized(state, request).await,
        TOOLS_LIST => handle_tools_list(state, request).await,
        TOOLS_CALL => handle_tools_call(state, request).await,
        _ => create_error_response(
            request.id,
            METHOD_NOT_FOUND,
            format!("Method not found: {}", request.method),
        ),
    }
}

async fn handle_initialize(_state: McpServerState, request: JsonRpcRequest) -> JsonRpcResponse {
    let response = InitializeResponse {
        protocol_version: "1.0".to_string(),
        capabilities: ServerCapabilities {
            tools: Some(ToolsCapability {}),
            resources: None,
            prompts: None,
        },
        server_info: Some(ServerInfo {
            name: "claude-utils-clipboard".to_string(),
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
        }),
    };

    create_success_response(request.id, serde_json::to_value(response).unwrap())
}

async fn handle_initialized(state: McpServerState, request: JsonRpcRequest) -> JsonRpcResponse {
    *state.initialized.write().await = true;
    info!("MCP server initialized");
    create_success_response(request.id, json!({}))
}

async fn handle_tools_list(_state: McpServerState, request: JsonRpcRequest) -> JsonRpcResponse {
    let tools = vec![
        Tool {
            name: "clipboard.get".to_string(),
            description: "Get current clipboard content (text or image)".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "format": {
                        "type": "string",
                        "enum": ["auto", "text", "image"],
                        "description": "Preferred format (auto detects automatically)",
                        "default": "auto"
                    }
                },
                "required": []
            }),
        },
        Tool {
            name: "clipboard.set".to_string(),
            description: "Set clipboard content (requires --write flag)".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "type": {
                        "type": "string",
                        "enum": ["text/plain", "image/png"],
                        "description": "Content type"
                    },
                    "data": {
                        "type": "string",
                        "description": "Content data (text or base64 for images)"
                    }
                },
                "required": ["type", "data"]
            }),
        },
    ];

    let response = ToolListResponse { tools };
    create_success_response(request.id, serde_json::to_value(response).unwrap())
}

async fn handle_tools_call(state: McpServerState, request: JsonRpcRequest) -> JsonRpcResponse {
    let params = match request.params {
        Some(p) => p,
        None => {
            return create_error_response(
                request.id,
                INVALID_PARAMS,
                "Missing parameters".to_string(),
            )
        }
    };

    let tool_request: ToolCallRequest = match serde_json::from_value(params) {
        Ok(r) => r,
        Err(e) => {
            return create_error_response(
                request.id,
                INVALID_PARAMS,
                format!("Invalid parameters: {e}"),
            )
        }
    };

    match tool_request.name.as_str() {
        "clipboard.get" => handle_clipboard_get(state, request.id, tool_request.arguments).await,
        "clipboard.set" => handle_clipboard_set(state, request.id, tool_request.arguments).await,
        _ => create_error_response(
            request.id,
            METHOD_NOT_FOUND,
            format!("Unknown tool: {}", tool_request.name),
        ),
    }
}

async fn handle_clipboard_get(
    state: McpServerState,
    id: Option<Value>,
    _args: Option<Value>,
) -> JsonRpcResponse {
    // Get clipboard content
    let clipboard_data = match state.clipboard.get_content() {
        Ok(data) => data,
        Err(e) => {
            return create_error_response(id, INTERNAL_ERROR, format!("Clipboard error: {e}"))
        }
    };

    // Handle image staging if needed
    let final_content = match &clipboard_data.content {
        ClipboardContent::ImagePng {
            data: None,
            width,
            height,
            size,
            ..
        }
        | ClipboardContent::ImageJpeg {
            data: None,
            width,
            height,
            size,
            ..
        } => {
            // Need to stage the image
            match state.clipboard.get_raw_image() {
                Ok(image_data) => {
                    match state.file_manager.stage_image(&image_data, "png").await {
                        Ok(staged) => {
                            // Update content with file path
                            match clipboard_data.content {
                                ClipboardContent::ImagePng { .. } => ClipboardContent::ImagePng {
                                    data: None,
                                    file: Some(staged.path.to_string_lossy().to_string()),
                                    width: *width,
                                    height: *height,
                                    size: *size,
                                },
                                ClipboardContent::ImageJpeg { .. } => ClipboardContent::ImageJpeg {
                                    data: None,
                                    file: Some(staged.path.to_string_lossy().to_string()),
                                    width: *width,
                                    height: *height,
                                    size: *size,
                                },
                                _ => clipboard_data.content.clone(),
                            }
                        }
                        Err(e) => {
                            error!("Failed to stage image: {}", e);
                            clipboard_data.content.clone()
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to get raw image: {}", e);
                    clipboard_data.content.clone()
                }
            }
        }
        _ => clipboard_data.content.clone(),
    };

    // Create response
    let response_data = json!({
        "content": final_content,
        "metadata": clipboard_data.metadata,
    });

    let tool_response = ToolCallResponse {
        content: vec![Content::Text {
            text: serde_json::to_string_pretty(&response_data).unwrap(),
        }],
    };

    create_success_response(id, serde_json::to_value(tool_response).unwrap())
}

async fn handle_clipboard_set(
    state: McpServerState,
    id: Option<Value>,
    args: Option<Value>,
) -> JsonRpcResponse {
    // TODO: Check for --write flag permission

    #[derive(Deserialize)]
    struct SetArgs {
        r#type: String,
        data: String,
    }

    let args: SetArgs = match args.and_then(|a| serde_json::from_value(a).ok()) {
        Some(a) => a,
        None => return create_error_response(id, INVALID_PARAMS, "Invalid arguments".to_string()),
    };

    let content = match args.r#type.as_str() {
        "text/plain" => ClipboardContent::Text {
            data: args.data,
            truncated: None,
        },
        "image/png" => ClipboardContent::ImagePng {
            data: Some(args.data),
            file: None,
            width: 0, // Will be updated by clipboard manager
            height: 0,
            size: 0,
        },
        _ => {
            return create_error_response(
                id,
                INVALID_PARAMS,
                format!("Unsupported type: {}", args.r#type),
            )
        }
    };

    match state.clipboard.set_content(&content) {
        Ok(_) => {
            let tool_response = ToolCallResponse {
                content: vec![Content::Text {
                    text: "Clipboard updated successfully".to_string(),
                }],
            };
            create_success_response(id, serde_json::to_value(tool_response).unwrap())
        }
        Err(e) => {
            create_error_response(id, INTERNAL_ERROR, format!("Failed to set clipboard: {e}"))
        }
    }
}

// SSE handler for real-time updates
async fn sse_handler(
    State(state): State<McpServerState>,
    Query(auth): Query<AuthQuery>,
) -> std::result::Result<impl IntoResponse, StatusCode> {
    // Check authentication
    if !state
        .auth_manager
        .validate_token(auth.token.as_deref())
        .await
    {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let stream = async_stream::stream! {
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;
            yield Ok::<_, anyhow::Error>(Event::default()
                .data("heartbeat")
                .event("ping"));
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
