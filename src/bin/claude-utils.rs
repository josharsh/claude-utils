use clap::{Parser, Subcommand};
use claude_utils::{
    clipboard::{
        processor::{ClipboardProcessor, ProcessorConfig},
        watcher::ClipboardWatcher,
        ClipboardManager,
    },
    file_manager::{FileManager, FileManagerConfig},
    mcp::{
        auth::{AuthConfig, AuthManager},
        server::McpServer,
    },
    Result, DEFAULT_HOST, DEFAULT_PORT,
};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(
    name = "claude-utils",
    about = "Cross-platform companion toolkit for Claude Code",
    version,
    author
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the clipboard daemon
    Start {
        /// Port to listen on
        #[arg(short, long, default_value_t = DEFAULT_PORT)]
        port: u16,

        /// Host to bind to
        #[arg(short = 'H', long, default_value = DEFAULT_HOST)]
        host: String,

        /// Disable authentication
        #[arg(long)]
        no_auth: bool,

        /// Custom staging directory
        #[arg(long)]
        staging_dir: Option<PathBuf>,

        /// Allow clipboard write operations
        #[arg(long)]
        write: bool,

        /// Enable clipboard watching mode
        #[arg(short, long)]
        watch: bool,

        /// Custom symlink directory (default: ~/Desktop)
        #[arg(long)]
        symlink_dir: Option<PathBuf>,

        /// Disable dual-format clipboard (path + image)
        #[arg(long)]
        no_dual_format: bool,

        /// Disable notifications
        #[arg(long)]
        no_notifications: bool,
    },

    /// Show authentication token
    Token,

    /// Generate MCP configuration
    Config {
        /// Output path for .mcp.json
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Quick clipboard operations
    Clip {
        #[command(subcommand)]
        action: ClipAction,
    },
}

#[derive(Subcommand)]
enum ClipAction {
    /// Get current clipboard content
    Get {
        /// Output format (json, text)
        #[arg(short, long, default_value = "json")]
        format: String,
    },

    /// Paste clipboard content (outputs path if image)
    Paste,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "claude_utils=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Start {
            port,
            host,
            no_auth,
            staging_dir,
            write,
            watch,
            symlink_dir,
            no_dual_format,
            no_notifications,
        } => {
            info!("Starting Claude-Utils clipboard daemon...");

            // Initialize components
            let clipboard = Arc::new(ClipboardManager::new()?);

            let file_config = if let Some(dir) = staging_dir {
                FileManagerConfig {
                    staging_dir: dir,
                    ..Default::default()
                }
            } else {
                FileManagerConfig::default()
            };

            let file_manager = Arc::new(FileManager::new(file_config).await?);

            let auth_config = AuthConfig {
                require_auth: !no_auth,
                ..Default::default()
            };

            let auth_manager = AuthManager::new(auth_config).await?;

            if let Some(token) = auth_manager.get_token().await {
                info!("Authentication token: {}", token);
                info!("Set CLAUDE_UTILS_TOKEN={} in your environment", token);
            }

            // Start clipboard watcher if enabled
            if watch {
                info!("Clipboard watching enabled");

                let processor_config = ProcessorConfig {
                    symlink_dir: symlink_dir.unwrap_or_else(|| {
                        dirs::home_dir()
                            .unwrap_or_else(|| PathBuf::from("."))
                            .join("Desktop")
                    }),
                    enable_dual_format: !no_dual_format,
                    enable_notifications: !no_notifications,
                    ..Default::default()
                };

                let (watcher, event_rx) = ClipboardWatcher::new(
                    clipboard.clone(),
                    Duration::from_millis(500), // Poll every 500ms
                );

                let processor = ClipboardProcessor::new(
                    processor_config,
                    file_manager.clone(),
                    clipboard.clone(),
                );

                // Spawn watcher task
                tokio::spawn(async move {
                    watcher.start_watching().await;
                });

                // Spawn processor task
                tokio::spawn(async move {
                    processor.start_processing(event_rx).await;
                });

                info!("Clipboard watcher started");
                info!("Images will be saved to Desktop with dual-format clipboard");
            }

            // Start server
            let server = McpServer::new(
                clipboard.clone(),
                file_manager.clone(),
                auth_manager,
                port,
                host.clone(),
            )
            .await?;

            info!("Starting MCP server on {}:{}", host, port);
            if write {
                info!("Write operations enabled");
            }

            server.run().await?;
        }

        Commands::Token => {
            let auth_manager = AuthManager::new(AuthConfig::default()).await?;

            if let Some(token) = auth_manager.get_token().await {
                println!("{token}");
            } else {
                error!("No authentication token found");
                std::process::exit(1);
            }
        }

        Commands::Config { output } => {
            let config = serde_json::json!({
                "claude-utils": {
                    "command": "claude-utils",
                    "args": ["start"],
                    "env": {
                        "CLAUDE_UTILS_TOKEN": "${CLAUDE_UTILS_TOKEN}"
                    }
                }
            });

            let config_str = serde_json::to_string_pretty(&config)?;

            if let Some(path) = output {
                std::fs::write(&path, config_str)?;
                info!("MCP configuration written to {}", path.display());
            } else {
                println!("{config_str}");
            }
        }

        Commands::Clip { action } => {
            let clipboard = ClipboardManager::new()?;

            match action {
                ClipAction::Get { format } => {
                    let content = clipboard.get_content()?;

                    match format.as_str() {
                        "json" => {
                            println!("{}", serde_json::to_string_pretty(&content)?);
                        }
                        "text" => match &content.content {
                            claude_utils::clipboard::ClipboardContent::Text { data, .. } => {
                                println!("{data}");
                            }
                            _ => {
                                println!("[Image in clipboard]");
                            }
                        },
                        _ => {
                            error!("Unknown format: {}", format);
                            std::process::exit(1);
                        }
                    }
                }

                ClipAction::Paste => {
                    let content = clipboard.get_content()?;

                    match &content.content {
                        claude_utils::clipboard::ClipboardContent::Text { data, .. } => {
                            print!("{data}");
                        }
                        claude_utils::clipboard::ClipboardContent::ImagePng { .. }
                        | claude_utils::clipboard::ClipboardContent::ImageJpeg { .. } => {
                            // Stage image and output path
                            let file_manager =
                                FileManager::new(FileManagerConfig::default()).await?;
                            let image_data = clipboard.get_raw_image()?;
                            let staged = file_manager.stage_image(&image_data, "png").await?;
                            print!("{}", staged.path.display());
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
