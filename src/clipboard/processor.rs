use chrono::Local;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use super::{
    watcher::{platform::DualClipboard, ClipboardEvent},
    ClipboardContent,
};
use crate::{file_manager::FileManager, Result};

#[derive(Debug, Clone)]
pub struct ProcessorConfig {
    pub symlink_dir: PathBuf,
    pub symlink_prefix: String,
    pub keep_symlinks: usize,
    pub enable_dual_format: bool,
    pub enable_notifications: bool,
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

        Self {
            symlink_dir: home.join("Desktop"),
            symlink_prefix: "claude-paste".to_string(),
            keep_symlinks: 5,
            enable_dual_format: true,
            enable_notifications: true,
        }
    }
}

pub struct ClipboardProcessor {
    config: ProcessorConfig,
    file_manager: Arc<FileManager>,
    clipboard_manager: Arc<super::ClipboardManager>,
}

impl ClipboardProcessor {
    pub fn new(
        config: ProcessorConfig,
        file_manager: Arc<FileManager>,
        clipboard_manager: Arc<super::ClipboardManager>,
    ) -> Self {
        Self {
            config,
            file_manager,
            clipboard_manager,
        }
    }

    pub async fn start_processing(self, mut event_rx: mpsc::Receiver<ClipboardEvent>) {
        info!("Clipboard processor started");

        while let Some(mut event) = event_rx.recv().await {
            if let Err(e) = self.process_event(&mut event).await {
                error!("Failed to process clipboard event: {}", e);
            }
        }
    }

    async fn process_event(&self, event: &mut ClipboardEvent) -> Result<()> {
        match &event.content.content {
            ClipboardContent::ImagePng { .. } | ClipboardContent::ImageJpeg { .. } => {
                self.process_image_event(event).await?;
            }
            ClipboardContent::Text { data, .. } if data.len() > crate::MAX_INLINE_SIZE => {
                self.process_large_text_event(event).await?;
            }
            _ => {
                // Small text passes through unchanged
                debug!("Small text content, no processing needed");
            }
        }

        Ok(())
    }

    async fn process_image_event(&self, event: &mut ClipboardEvent) -> Result<()> {
        info!("Processing image clipboard event");

        // Get raw image data
        let image_data = self.clipboard_manager.get_raw_image()?;

        // Stage the image
        let format = match &event.content.content {
            ClipboardContent::ImagePng { .. } => "png",
            ClipboardContent::ImageJpeg { .. } => "jpeg",
            _ => unreachable!(),
        };

        let staged = self.file_manager.stage_image(&image_data, format).await?;
        event.staged_path = Some(staged.path.clone());

        // Create timestamped symlink
        let symlink_path = self.create_symlink(&staged.path, format).await?;
        event.symlink_path = Some(symlink_path.clone());

        // Set dual clipboard if enabled
        if self.config.enable_dual_format {
            let path_str = symlink_path.to_string_lossy();

            #[cfg(target_os = "macos")]
            {
                if let Err(e) = DualClipboard::set_dual_content(&path_str, &image_data) {
                    warn!("Failed to set dual clipboard format: {}", e);
                    // Fallback to text-only
                    self.set_text_clipboard(&path_str)?;
                } else {
                    info!("Set dual clipboard: text path + original image");
                }
            }

            #[cfg(not(target_os = "macos"))]
            {
                // On other platforms, just set text
                self.set_text_clipboard(&path_str)?;
            }
        }

        // Clean up old symlinks
        self.cleanup_old_symlinks().await?;

        // Show notification if enabled
        if self.config.enable_notifications {
            let notification_path = symlink_path.to_string_lossy();
            self.show_notification("Image ready for Claude Code", &notification_path);
        }

        info!("Image processed: {}", symlink_path.display());
        Ok(())
    }

    async fn process_large_text_event(&self, event: &mut ClipboardEvent) -> Result<()> {
        info!("Processing large text clipboard event");

        if let ClipboardContent::Text { data, .. } = &event.content.content {
            // Stage the text
            let staged = self.file_manager.stage_text(data).await?;
            event.staged_path = Some(staged.path.clone());

            // Create symlink
            let symlink_path = self.create_symlink(&staged.path, "txt").await?;
            event.symlink_path = Some(symlink_path.clone());

            // Update clipboard with path
            let path_str = symlink_path.to_string_lossy();
            self.set_text_clipboard(&path_str)?;

            // Clean up old symlinks
            self.cleanup_old_symlinks().await?;

            info!("Large text processed: {}", symlink_path.display());
        }

        Ok(())
    }

    async fn create_symlink(&self, target: &Path, extension: &str) -> Result<PathBuf> {
        // Generate timestamped filename
        let timestamp = Local::now().format("%Y%m%d-%H%M%S");
        let filename = format!("{}-{}.{}", self.config.symlink_prefix, timestamp, extension);
        let symlink_path = self.config.symlink_dir.join(&filename);

        // Create symlink
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(target, &symlink_path)?;
        }

        #[cfg(windows)]
        {
            use std::os::windows::fs::symlink_file;
            symlink_file(target, &symlink_path)?;
        }

        // Also create a "latest" symlink for convenience
        let latest_name = format!("{}.{}", self.config.symlink_prefix, extension);
        let latest_path = self.config.symlink_dir.join(&latest_name);

        // Remove old latest symlink if exists
        let _ = fs::remove_file(&latest_path).await;

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(target, &latest_path)?;
        }

        #[cfg(windows)]
        {
            use std::os::windows::fs::symlink_file;
            symlink_file(target, &latest_path)?;
        }

        Ok(symlink_path)
    }

    async fn cleanup_old_symlinks(&self) -> Result<()> {
        let pattern = format!("{}-*", self.config.symlink_prefix);
        let mut entries = fs::read_dir(&self.config.symlink_dir).await?;
        let mut symlinks = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with(&pattern) && name.contains('-') {
                    if let Ok(metadata) = entry.metadata().await {
                        if metadata.file_type().is_symlink() {
                            if let Ok(modified) = metadata.modified() {
                                symlinks.push((path, modified));
                            }
                        }
                    }
                }
            }
        }

        // Sort by modification time (newest first)
        symlinks.sort_by(|a, b| b.1.cmp(&a.1));

        // Remove old symlinks beyond keep limit
        for (path, _) in symlinks.into_iter().skip(self.config.keep_symlinks) {
            if let Err(e) = fs::remove_file(&path).await {
                warn!("Failed to remove old symlink: {}", e);
            } else {
                debug!("Removed old symlink: {}", path.display());
            }
        }

        Ok(())
    }

    fn set_text_clipboard(&self, text: &str) -> Result<()> {
        self.clipboard_manager.set_content(&ClipboardContent::Text {
            data: text.to_string(),
            truncated: None,
        })
    }

    fn show_notification(&self, title: &str, body: &str) {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            let script = format!(
                r#"display notification "{body}" with title "Claude-Utils" subtitle "{title}""#
            );
            let _ = Command::new("osascript").arg("-e").arg(&script).output();
        }

        #[cfg(target_os = "linux")]
        {
            use std::process::Command;
            let _ = Command::new("notify-send")
                .arg("Claude-Utils")
                .arg(&format!("{}\n{}", title, body))
                .output();
        }

        #[cfg(target_os = "windows")]
        {
            // Windows notifications require more setup, skip for now
            info!("Notification: {} - {}", title, body);
        }
    }
}

use tracing::debug;
