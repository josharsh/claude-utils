use rand::Rng;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::Result;

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub token_path: PathBuf,
    pub require_auth: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        let token_path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".claude-utils")
            .join("auth.token");

        Self {
            token_path,
            require_auth: true,
        }
    }
}

pub struct AuthManager {
    config: AuthConfig,
    token: Arc<RwLock<Option<String>>>,
}

impl AuthManager {
    pub async fn new(config: AuthConfig) -> Result<Self> {
        let manager = Self {
            config,
            token: Arc::new(RwLock::new(None)),
        };

        manager.initialize().await?;
        Ok(manager)
    }

    async fn initialize(&self) -> Result<()> {
        if !self.config.require_auth {
            return Ok(());
        }

        // Ensure directory exists
        if let Some(parent) = self.config.token_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Load or generate token
        let token = if self.config.token_path.exists() {
            fs::read_to_string(&self.config.token_path)?
                .trim()
                .to_string()
        } else {
            let new_token = self.generate_token();
            self.save_token(&new_token)?;
            new_token
        };

        *self.token.write().await = Some(token);
        Ok(())
    }

    fn generate_token(&self) -> String {
        let mut rng = rand::thread_rng();
        let token_bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        hex::encode(token_bytes)
    }

    fn save_token(&self, token: &str) -> Result<()> {
        fs::write(&self.config.token_path, token)?;

        // Set restrictive permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = fs::Permissions::from_mode(0o600);
            fs::set_permissions(&self.config.token_path, permissions)?;
        }

        Ok(())
    }

    pub async fn validate_token(&self, provided_token: Option<&str>) -> bool {
        if !self.config.require_auth {
            return true;
        }

        let stored_token = self.token.read().await;
        match (&*stored_token, provided_token) {
            (Some(stored), Some(provided)) => stored == provided,
            _ => false,
        }
    }

    pub async fn get_token(&self) -> Option<String> {
        self.token.read().await.clone()
    }
}

// Hex encoding utility
mod hex {
    pub fn encode(bytes: Vec<u8>) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()
    }
}
