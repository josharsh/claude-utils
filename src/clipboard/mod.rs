pub mod processor;
pub mod watcher;

use arboard::{Clipboard as Arboard, ImageData};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::{ClaudeUtilsError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClipboardContent {
    #[serde(rename = "text/plain")]
    Text {
        data: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        truncated: Option<bool>,
    },
    #[serde(rename = "image/png")]
    ImagePng {
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<String>, // base64 encoded if small
        #[serde(skip_serializing_if = "Option::is_none")]
        file: Option<String>, // file path if large
        width: usize,
        height: usize,
        size: usize,
    },
    #[serde(rename = "image/jpeg")]
    ImageJpeg {
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        file: Option<String>,
        width: usize,
        height: usize,
        size: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardMetadata {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardData {
    #[serde(flatten)]
    pub content: ClipboardContent,
    pub metadata: ClipboardMetadata,
}

pub struct ClipboardManager {
    clipboard: Arc<Mutex<Arboard>>,
}

impl ClipboardManager {
    pub fn new() -> Result<Self> {
        let clipboard = Arboard::new().map_err(|e| ClaudeUtilsError::Clipboard(e.to_string()))?;

        Ok(Self {
            clipboard: Arc::new(Mutex::new(clipboard)),
        })
    }

    pub fn get_content(&self) -> Result<ClipboardData> {
        let mut clipboard = self
            .clipboard
            .lock()
            .map_err(|e| ClaudeUtilsError::Clipboard(format!("Lock error: {e}")))?;

        // Try to get image first (more specific)
        if let Ok(image_data) = clipboard.get_image() {
            return self.process_image(image_data);
        }

        // Fall back to text
        if let Ok(text) = clipboard.get_text() {
            return Ok(self.process_text(text));
        }

        Err(ClaudeUtilsError::Clipboard(
            "No content in clipboard".to_string(),
        ))
    }

    pub fn set_content(&self, content: &ClipboardContent) -> Result<()> {
        let mut clipboard = self
            .clipboard
            .lock()
            .map_err(|e| ClaudeUtilsError::Clipboard(format!("Lock error: {e}")))?;

        match content {
            ClipboardContent::Text { data, .. } => {
                clipboard
                    .set_text(data)
                    .map_err(|e| ClaudeUtilsError::Clipboard(e.to_string()))?;
            }
            ClipboardContent::ImagePng {
                data: Some(base64_data),
                width,
                height,
                ..
            }
            | ClipboardContent::ImageJpeg {
                data: Some(base64_data),
                width,
                height,
                ..
            } => {
                let bytes = BASE64.decode(base64_data).map_err(|e| {
                    ClaudeUtilsError::Clipboard(format!("Base64 decode error: {e}"))
                })?;

                let image_data = ImageData {
                    width: *width,
                    height: *height,
                    bytes: bytes.into(),
                };

                clipboard
                    .set_image(image_data)
                    .map_err(|e| ClaudeUtilsError::Clipboard(e.to_string()))?;
            }
            _ => {
                return Err(ClaudeUtilsError::Clipboard(
                    "Cannot set clipboard from file reference".to_string(),
                ));
            }
        }

        Ok(())
    }

    fn process_text(&self, text: String) -> ClipboardData {
        let truncated = text.len() > crate::MAX_INLINE_SIZE;
        let data = if truncated {
            text.chars().take(crate::MAX_INLINE_SIZE).collect()
        } else {
            text.clone()
        };

        ClipboardData {
            content: ClipboardContent::Text {
                data,
                truncated: if truncated { Some(true) } else { None },
            },
            metadata: ClipboardMetadata {
                timestamp: chrono::Utc::now(),
                source: None,
            },
        }
    }

    fn process_image(&self, image_data: ImageData<'_>) -> Result<ClipboardData> {
        use image::{ImageFormat, RgbaImage};

        // Convert arboard image data to image crate format
        let img = RgbaImage::from_raw(
            image_data.width as u32,
            image_data.height as u32,
            image_data.bytes.to_vec(),
        )
        .ok_or_else(|| {
            ClaudeUtilsError::ImageProcessing(image::ImageError::Limits(
                image::error::LimitError::from_kind(image::error::LimitErrorKind::DimensionError),
            ))
        })?;

        // Detect format and encode
        let mut png_bytes = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut png_bytes), ImageFormat::Png)?;

        let size = png_bytes.len();
        let (data, file) = if size <= crate::MAX_INLINE_SIZE {
            (Some(BASE64.encode(&png_bytes)), None)
        } else {
            // Will be handled by file manager
            (None, None)
        };

        Ok(ClipboardData {
            content: ClipboardContent::ImagePng {
                data,
                file,
                width: image_data.width,
                height: image_data.height,
                size,
            },
            metadata: ClipboardMetadata {
                timestamp: chrono::Utc::now(),
                source: None,
            },
        })
    }

    pub fn get_raw_image(&self) -> Result<Vec<u8>> {
        let mut clipboard = self
            .clipboard
            .lock()
            .map_err(|e| ClaudeUtilsError::Clipboard(format!("Lock error: {e}")))?;

        let image_data = clipboard
            .get_image()
            .map_err(|e| ClaudeUtilsError::Clipboard(e.to_string()))?;

        // Convert to PNG
        let img = image::RgbaImage::from_raw(
            image_data.width as u32,
            image_data.height as u32,
            image_data.bytes.to_vec(),
        )
        .ok_or_else(|| {
            ClaudeUtilsError::ImageProcessing(image::ImageError::Limits(
                image::error::LimitError::from_kind(image::error::LimitErrorKind::DimensionError),
            ))
        })?;

        let mut png_bytes = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut png_bytes),
            image::ImageFormat::Png,
        )?;

        Ok(png_bytes)
    }
}
