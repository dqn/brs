// Table updater — fetches difficulty tables from HTTP and updates local cache.
//
// Downloads table header + data JSON from configured URLs, parses them
// with `difficulty_table_parser`, and writes the result to the local
// cache via `TableDataAccessor`.

use anyhow::Result;
use tracing::{info, warn};

use bms_database::difficulty_table_parser::{
    extract_bmstable_url, parse_json_data, parse_json_header, resolve_url, to_table_data,
};
use bms_database::{TableData, TableDataAccessor};

/// Fetch and parse a single table from its URL, returning the resulting `TableData`.
///
/// The URL may point to:
/// - An HTML page containing a `<meta name="bmstable">` tag
/// - A JSON header directly
///
/// After parsing the header, all `data_url` entries are fetched and merged.
async fn fetch_and_parse(client: &reqwest::Client, url: &str) -> Result<TableData> {
    // Fetch the initial URL
    let response = client.get(url).send().await?.text().await?;

    // Determine the header URL: either extracted from HTML meta tag, or the URL itself
    let (header_url, header_json) = if let Some(meta_url) = extract_bmstable_url(&response) {
        let resolved = resolve_url(url, &meta_url);
        let json = client.get(&resolved).send().await?.text().await?;
        (resolved, json)
    } else {
        // Assume the response is the JSON header directly
        (url.to_string(), response)
    };

    let header = parse_json_header(&header_json)?;

    // Fetch all data URLs and merge charts
    let mut all_charts = Vec::new();
    for data_url in &header.data_url {
        let resolved = resolve_url(&header_url, data_url);
        match client.get(&resolved).send().await {
            Ok(resp) => match resp.text().await {
                Ok(text) => match parse_json_data(&text) {
                    Ok(charts) => all_charts.extend(charts),
                    Err(e) => warn!(url = %resolved, "Failed to parse table data: {e}"),
                },
                Err(e) => warn!(url = %resolved, "Failed to read table data response: {e}"),
            },
            Err(e) => warn!(url = %resolved, "Failed to fetch table data: {e}"),
        }
    }

    Ok(to_table_data(&header, &all_charts, url))
}

/// Update all tables from the given URLs, writing results to the cache directory.
///
/// Fetches all URLs concurrently using a JoinSet. Errors for individual tables
/// are logged but do not prevent other tables from being updated (partial success).
///
/// Returns the list of successfully updated `TableData`.
pub async fn update_all(urls: &[String], table_dir: &str) -> Vec<TableData> {
    let accessor = match TableDataAccessor::new(table_dir) {
        Ok(a) => a,
        Err(e) => {
            warn!("Failed to create table accessor: {e}");
            return Vec::new();
        }
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_default();

    let mut set = tokio::task::JoinSet::new();
    for url in urls {
        let client = client.clone();
        let url = url.clone();
        set.spawn(async move {
            match fetch_and_parse(&client, &url).await {
                Ok(td) => {
                    info!(name = %td.name, url = %url, "Table fetched successfully");
                    Some(td)
                }
                Err(e) => {
                    warn!(url = %url, "Failed to fetch/parse table: {e}");
                    None
                }
            }
        });
    }

    let mut updated = Vec::new();
    while let Some(result) = set.join_next().await {
        if let Ok(Some(td)) = result {
            if let Err(e) = accessor.write(&td) {
                warn!(name = %td.name, "Failed to write table cache: {e}");
            } else {
                updated.push(td);
            }
        }
    }

    info!(
        count = updated.len(),
        total = urls.len(),
        "Table update complete"
    );
    updated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn update_all_empty_urls() {
        let dir = tempfile::tempdir().unwrap();
        let result = update_all(&[], dir.path().to_str().unwrap()).await;
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn update_all_invalid_url_logs_error() {
        let dir = tempfile::tempdir().unwrap();
        let urls = vec!["http://127.0.0.1:1/nonexistent".to_string()];
        let result = update_all(&urls, dir.path().to_str().unwrap()).await;
        assert!(result.is_empty());
    }
}
