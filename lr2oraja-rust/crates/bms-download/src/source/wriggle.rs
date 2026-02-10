// Wriggle download source
//
// Simple URL-based download source that substitutes the hash directly
// into the URL pattern. No metadata query needed.
// Default: https://bms.wrigglebug.xyz/download/package/{md5}

use super::DownloadSource;

const DEFAULT_URL: &str = "https://bms.wrigglebug.xyz/download/package/{}";

/// Wriggle download source with direct URL pattern substitution.
pub struct WriggleDownloadSource {
    /// URL pattern with `{}` placeholder for the hash.
    download_url: String,
}

impl WriggleDownloadSource {
    pub fn new(override_url: Option<&str>) -> Self {
        let download_url = override_url
            .filter(|u| !u.is_empty())
            .map(String::from)
            .unwrap_or_else(|| DEFAULT_URL.to_string());
        Self { download_url }
    }

    /// Return the default download URL pattern.
    pub fn default_url() -> &'static str {
        DEFAULT_URL
    }
}

impl DownloadSource for WriggleDownloadSource {
    fn name(&self) -> &str {
        "wriggle"
    }

    async fn get_download_url(&self, hash: &str) -> anyhow::Result<String> {
        Ok(self.download_url.replace("{}", hash))
    }

    fn allow_md5(&self) -> bool {
        true
    }

    fn allow_sha256(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_url() {
        let source = WriggleDownloadSource::new(None);
        assert_eq!(source.download_url, DEFAULT_URL);
        assert_eq!(source.name(), "wriggle");
    }

    #[test]
    fn test_override_url() {
        let source = WriggleDownloadSource::new(Some("https://custom.example.com/dl/{}"));
        assert_eq!(source.download_url, "https://custom.example.com/dl/{}");
    }

    #[test]
    fn test_empty_override_uses_default() {
        let source = WriggleDownloadSource::new(Some(""));
        assert_eq!(source.download_url, DEFAULT_URL);
    }

    #[test]
    fn test_allow_md5() {
        let source = WriggleDownloadSource::new(None);
        assert!(source.allow_md5());
        assert!(!source.allow_sha256());
    }

    #[tokio::test]
    async fn test_url_generation() {
        let source = WriggleDownloadSource::new(None);
        let url = source.get_download_url("abc123def456").await.unwrap();
        assert_eq!(
            url,
            "https://bms.wrigglebug.xyz/download/package/abc123def456"
        );
    }

    #[tokio::test]
    async fn test_url_generation_custom() {
        let source = WriggleDownloadSource::new(Some("https://mirror.example.com/bms/{}"));
        let url = source.get_download_url("test_hash").await.unwrap();
        assert_eq!(url, "https://mirror.example.com/bms/test_hash");
    }
}
