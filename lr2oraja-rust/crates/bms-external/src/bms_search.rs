use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

const BMS_SEARCH_API_BASE: &str = "https://api.bmssearch.net/v1";

/// A BMS search result entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BmsSearchEntry {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub genre: String,
    #[serde(default)]
    pub artist: String,
    #[serde(default)]
    pub url: String,
}

/// BMS search API response wrapper.
#[derive(Debug, Clone, Deserialize)]
struct SearchResponse {
    #[serde(default)]
    data: Vec<BmsSearchEntry>,
}

/// BMS Search API accessor.
pub struct BmsSearchAccessor {
    client: reqwest::Client,
}

impl BmsSearchAccessor {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Search for BMS entries by query string.
    pub async fn search(&self, query: &str) -> Result<Vec<BmsSearchEntry>> {
        let url = format!("{}/bmses/search", BMS_SEARCH_API_BASE);
        info!("BMS search: querying '{}'", query);

        let response = self.client.get(&url).query(&[("q", query)]).send().await?;

        let status = response.status();
        if !status.is_success() {
            anyhow::bail!("BMS search API returned status {}", status);
        }

        let body: SearchResponse = response.json().await?;
        Ok(body.data)
    }

    /// Get MD5 hashes for BMS files at a given URL.
    pub async fn get_md5s_by_url(&self, url: &str) -> Result<Vec<String>> {
        let api_url = format!("{}/bmses/search", BMS_SEARCH_API_BASE);
        info!("BMS search: looking up MD5s for URL '{}'", url);

        let response = self
            .client
            .get(&api_url)
            .query(&[("url", url)])
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            anyhow::bail!("BMS search API returned status {}", status);
        }

        let body: SearchResponse = response.json().await?;
        // Extract any MD5 hashes from search results (the API returns BMS entries)
        Ok(body.data.into_iter().map(|e| e.id).collect())
    }
}

impl Default for BmsSearchAccessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bms_search_entry_deserialize() {
        let json = r#"{"id":"abc123","title":"Test BMS","genre":"Techno","artist":"DJ Test","url":"https://example.com/bms.zip"}"#;
        let entry: BmsSearchEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.id, "abc123");
        assert_eq!(entry.title, "Test BMS");
        assert_eq!(entry.genre, "Techno");
        assert_eq!(entry.artist, "DJ Test");
        assert_eq!(entry.url, "https://example.com/bms.zip");
    }

    #[test]
    fn bms_search_entry_missing_optional_fields() {
        let json = r#"{"id":"abc","title":"Minimal"}"#;
        let entry: BmsSearchEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.id, "abc");
        assert_eq!(entry.title, "Minimal");
        assert!(entry.genre.is_empty());
        assert!(entry.artist.is_empty());
        assert!(entry.url.is_empty());
    }

    #[test]
    fn search_response_deserialize() {
        let json = r#"{"data":[{"id":"1","title":"A","genre":"","artist":"","url":""},{"id":"2","title":"B","genre":"","artist":"","url":""}]}"#;
        let resp: SearchResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.data.len(), 2);
    }

    #[test]
    fn search_response_empty() {
        let json = r#"{"data":[]}"#;
        let resp: SearchResponse = serde_json::from_str(json).unwrap();
        assert!(resp.data.is_empty());
    }

    #[test]
    fn bms_search_accessor_default() {
        let _accessor = BmsSearchAccessor::default();
    }

    #[test]
    fn bms_search_entry_serde_round_trip() {
        let entry = BmsSearchEntry {
            id: "test-id".to_string(),
            title: "Test Title".to_string(),
            genre: "Genre".to_string(),
            artist: "Artist".to_string(),
            url: "https://example.com".to_string(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: BmsSearchEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, entry.id);
        assert_eq!(deserialized.title, entry.title);
    }
}
