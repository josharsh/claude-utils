// Test that all modules compile correctly
#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_modules_exist() {
        // Just verify modules are accessible
        let _ = clipboard::ClipboardManager::new();
        let _ = file_manager::FileManagerConfig::default();
        let _ = mcp::auth::AuthConfig::default();
    }

    #[test]
    fn test_constants() {
        assert_eq!(DEFAULT_PORT, 3830);
        assert_eq!(DEFAULT_HOST, "127.0.0.1");
        assert_eq!(MAX_INLINE_SIZE, 65536);
    }
}
