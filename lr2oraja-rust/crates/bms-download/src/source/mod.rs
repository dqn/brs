// Download source definitions
//
// Defines the DownloadSource trait and concrete implementations
// for Konmai and Wriggle download services.

pub mod konmai;
pub mod wriggle;

/// Trait for BMS package download sources.
pub trait DownloadSource: Send + Sync {
    /// Human-readable name of the download source.
    fn name(&self) -> &str;

    /// Resolve the download URL for a given hash.
    fn get_download_url(
        &self,
        hash: &str,
    ) -> impl std::future::Future<Output = anyhow::Result<String>> + Send;

    /// Whether this source supports MD5-based lookups.
    fn allow_md5(&self) -> bool;

    /// Whether this source supports SHA256-based lookups.
    fn allow_sha256(&self) -> bool;
}
