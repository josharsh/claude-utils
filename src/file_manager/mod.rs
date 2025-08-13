use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use tokio::fs;
use tracing::{error, info, warn};

use crate::Result;

#[derive(Debug, Clone)]
pub struct StagedFile {
    pub path: PathBuf,
    pub size: usize,
    pub format: String,
    pub created_at: SystemTime,
    pub thumbnail_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct FileManagerConfig {
    pub staging_dir: PathBuf,
    pub cleanup_interval: Duration,
    pub max_file_age: Duration,
}

impl Default for FileManagerConfig {
    fn default() -> Self {
        let staging_dir = std::env::temp_dir().join(crate::STAGING_DIR_NAME);

        Self {
            staging_dir,
            cleanup_interval: Duration::from_secs(crate::CLEANUP_INTERVAL_MINS * 60),
            max_file_age: Duration::from_secs(crate::CLEANUP_INTERVAL_MINS * 60),
        }
    }
}

pub struct FileManager {
    config: FileManagerConfig,
    cache: Arc<Mutex<HashMap<String, StagedFile>>>,
}

impl FileManager {
    pub async fn new(config: FileManagerConfig) -> Result<Self> {
        // Ensure staging directory exists
        fs::create_dir_all(&config.staging_dir).await?;

        let manager = Self {
            config,
            cache: Arc::new(Mutex::new(HashMap::new())),
        };

        // Start cleanup task
        manager.start_cleanup_task();

        Ok(manager)
    }

    pub async fn stage_image(&self, data: &[u8], format: &str) -> Result<StagedFile> {
        // Calculate hash for deduplication
        let hash = self.calculate_hash(data);
        let filename = format!("clip-{}.{}", &hash[..8], format);
        let file_path = self.config.staging_dir.join(&filename);

        // Check cache first
        if let Some(staged) = self.get_from_cache(&hash) {
            if file_path.exists() {
                info!("Using cached file: {}", file_path.display());
                return Ok(staged);
            }
        }

        // Write main file
        fs::write(&file_path, data).await?;
        info!(
            "Staged file: {} ({} bytes)",
            file_path.display(),
            data.len()
        );

        // Generate thumbnail
        let thumbnail_path = self.generate_thumbnail(&file_path, data, format).await?;

        let staged_file = StagedFile {
            path: file_path,
            size: data.len(),
            format: format.to_string(),
            created_at: SystemTime::now(),
            thumbnail_path,
        };

        // Update cache
        self.update_cache(hash, staged_file.clone());

        Ok(staged_file)
    }

    pub async fn stage_text(&self, text: &str) -> Result<StagedFile> {
        let data = text.as_bytes();
        let hash = self.calculate_hash(data);
        let filename = format!("clip-{}.txt", &hash[..8]);
        let file_path = self.config.staging_dir.join(&filename);

        // Check cache
        if let Some(staged) = self.get_from_cache(&hash) {
            if file_path.exists() {
                return Ok(staged);
            }
        }

        // Write file
        fs::write(&file_path, text).await?;

        let staged_file = StagedFile {
            path: file_path,
            size: data.len(),
            format: "txt".to_string(),
            created_at: SystemTime::now(),
            thumbnail_path: None,
        };

        self.update_cache(hash, staged_file.clone());

        Ok(staged_file)
    }

    async fn generate_thumbnail(
        &self,
        file_path: &Path,
        data: &[u8],
        format: &str,
    ) -> Result<Option<PathBuf>> {
        use image::imageops::FilterType;

        // Only generate thumbnails for supported image formats
        if !["png", "jpg", "jpeg", "gif", "webp", "bmp"].contains(&format) {
            return Ok(None);
        }

        let thumb_path = file_path.with_extension("thumb.png");

        // Load and resize image
        match image::load_from_memory(data) {
            Ok(img) => {
                let thumbnail = img.resize(256, 256, FilterType::Lanczos3);

                // Save thumbnail
                match thumbnail.save(&thumb_path) {
                    Ok(_) => {
                        info!("Generated thumbnail: {}", thumb_path.display());
                        Ok(Some(thumb_path))
                    }
                    Err(e) => {
                        warn!("Failed to save thumbnail: {}", e);
                        Ok(None)
                    }
                }
            }
            Err(e) => {
                warn!("Failed to generate thumbnail: {}", e);
                Ok(None)
            }
        }
    }

    fn calculate_hash(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    fn get_from_cache(&self, hash: &str) -> Option<StagedFile> {
        self.cache.lock().ok()?.get(hash).cloned()
    }

    fn update_cache(&self, hash: String, file: StagedFile) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(hash, file);
        }
    }

    fn start_cleanup_task(&self) {
        let cache = self.cache.clone();
        let staging_dir = self.config.staging_dir.clone();
        let max_age = self.config.max_file_age;
        let interval = self.config.cleanup_interval;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                info!("Running cleanup task");

                // Clean up old files
                match fs::read_dir(&staging_dir).await {
                    Ok(mut entries) => {
                        while let Ok(Some(entry)) = entries.next_entry().await {
                            if let Ok(metadata) = entry.metadata().await {
                                if let Ok(modified) = metadata.modified() {
                                    if let Ok(age) = modified.elapsed() {
                                        if age > max_age {
                                            let path = entry.path();
                                            match fs::remove_file(&path).await {
                                                Ok(_) => {
                                                    info!("Cleaned up old file: {}", path.display())
                                                }
                                                Err(e) => warn!("Failed to remove file: {}", e),
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => error!("Failed to read staging directory: {}", e),
                }

                // Clean cache
                if let Ok(mut cache_guard) = cache.lock() {
                    let _now = SystemTime::now();
                    cache_guard.retain(|_, file| {
                        file.created_at
                            .elapsed()
                            .map(|age| age < max_age)
                            .unwrap_or(false)
                    });
                }
            }
        });
    }

    pub fn get_staging_dir(&self) -> &Path {
        &self.config.staging_dir
    }
}
