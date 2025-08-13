use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, MissedTickBehavior};
use tracing::{debug, error, info, warn};

use super::{ClipboardContent, ClipboardData, ClipboardManager};
use crate::Result;

#[derive(Debug, Clone, PartialEq)]
pub struct WatchedContent {
    pub content_hash: String,
    pub timestamp: SystemTime,
    pub content_type: ContentType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContentType {
    Text(usize),                 // size
    Image(String, usize, usize), // format, width, height
}

#[derive(Debug, Clone)]
pub struct ClipboardEvent {
    pub content: ClipboardData,
    pub staged_path: Option<PathBuf>,
    pub symlink_path: Option<PathBuf>,
}

pub struct ClipboardWatcher {
    clipboard: Arc<ClipboardManager>,
    last_content: Arc<RwLock<Option<WatchedContent>>>,
    poll_interval: Duration,
    event_sender: mpsc::Sender<ClipboardEvent>,
}

impl ClipboardWatcher {
    pub fn new(
        clipboard: Arc<ClipboardManager>,
        poll_interval: Duration,
    ) -> (Self, mpsc::Receiver<ClipboardEvent>) {
        let (tx, rx) = mpsc::channel(100);

        let watcher = Self {
            clipboard,
            last_content: Arc::new(RwLock::new(None)),
            poll_interval,
            event_sender: tx,
        };

        (watcher, rx)
    }

    pub async fn start_watching(self) {
        let mut interval_timer = interval(self.poll_interval);
        interval_timer.set_missed_tick_behavior(MissedTickBehavior::Skip);

        info!(
            "Clipboard watcher started (poll interval: {:?})",
            self.poll_interval
        );

        loop {
            interval_timer.tick().await;

            if let Err(e) = self.check_clipboard().await {
                error!("Clipboard check error: {}", e);
                // Continue watching despite errors
            }
        }
    }

    async fn check_clipboard(&self) -> Result<()> {
        // Get current clipboard content
        let current_data = match self.clipboard.get_content() {
            Ok(data) => data,
            Err(e) => {
                debug!("No clipboard content or error: {}", e);
                return Ok(());
            }
        };

        // Calculate content hash
        let content_hash = self.calculate_content_hash(&current_data.content);
        let content_type = self.get_content_type(&current_data.content);

        // Check if content changed
        let mut last = self.last_content.write().await;

        let changed = match &*last {
            Some(prev) => prev.content_hash != content_hash,
            None => true,
        };

        if !changed {
            return Ok(());
        }

        // Update last content
        *last = Some(WatchedContent {
            content_hash: content_hash.clone(),
            timestamp: SystemTime::now(),
            content_type: content_type.clone(),
        });
        drop(last); // Release write lock

        // Emit event for new content
        info!("New clipboard content detected: {:?}", content_type);

        let event = ClipboardEvent {
            content: current_data,
            staged_path: None,
            symlink_path: None,
        };

        if let Err(e) = self.event_sender.send(event).await {
            warn!("Failed to send clipboard event: {}", e);
        }

        Ok(())
    }

    fn calculate_content_hash(&self, content: &ClipboardContent) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();

        match content {
            ClipboardContent::Text { data, .. } => {
                hasher.update(b"text:");
                hasher.update(data.as_bytes());
            }
            ClipboardContent::ImagePng {
                data,
                file,
                width,
                height,
                size,
            }
            | ClipboardContent::ImageJpeg {
                data,
                file,
                width,
                height,
                size,
            } => {
                hasher.update(b"image:");
                hasher.update(width.to_le_bytes());
                hasher.update(height.to_le_bytes());
                hasher.update(size.to_le_bytes());

                if let Some(data) = data {
                    hasher.update(data.as_bytes());
                } else if let Some(file) = file {
                    hasher.update(file.as_bytes());
                }
            }
        }

        format!("{:x}", hasher.finalize())
    }

    fn get_content_type(&self, content: &ClipboardContent) -> ContentType {
        match content {
            ClipboardContent::Text { data, .. } => ContentType::Text(data.len()),
            ClipboardContent::ImagePng { width, height, .. } => {
                ContentType::Image("png".to_string(), *width, *height)
            }
            ClipboardContent::ImageJpeg { width, height, .. } => {
                ContentType::Image("jpeg".to_string(), *width, *height)
            }
        }
    }
}

// Platform-specific clipboard manager that can handle dual formats
#[cfg(target_os = "macos")]
pub mod platform {
    use super::*;

    pub struct DualClipboard;

    impl DualClipboard {
        /// Sets both text (file path) and image data in clipboard
        /// Terminal apps will get the text, image apps will get the image
        pub fn set_dual_content(path: &str, _image_data: &[u8]) -> Result<()> {
            // For now, let's use a simpler approach that definitely works
            // We'll just set the text path, and document that dual format
            // requires more complex macOS integration

            let clipboard = ClipboardManager::new()?;
            clipboard.set_content(&ClipboardContent::Text {
                data: path.to_string(),
                truncated: None,
            })?;

            warn!("Dual clipboard format not fully implemented on macOS yet");
            Ok(())
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub mod platform {
    use super::*;

    pub struct DualClipboard;

    impl DualClipboard {
        pub fn set_dual_content(path: &str, _image_data: &[u8]) -> Result<()> {
            // On other platforms, we'll just set the path as text
            // This is a fallback - could implement X11/Win32 specific code
            warn!("Dual clipboard not fully implemented for this platform");

            let clipboard = ClipboardManager::new()?;
            clipboard.set_content(&ClipboardContent::Text {
                data: path.to_string(),
                truncated: None,
            })
        }
    }
}
