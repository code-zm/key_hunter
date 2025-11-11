use crate::core::error::{KeyHunterError, Result};
use crate::core::results::{SearchQuery, SearchResult};
use crate::core::traits::SearchProvider;
use crate::utils::{HttpClient, RateLimiter};
use async_trait::async_trait;
use serde::Deserialize;
use std::time::Duration;
use tracing::{debug, info, warn};

#[derive(Debug, Deserialize)]
struct GitHubSearchResponse {
    total_count: u64,
    items: Vec<GitHubSearchItem>,
}

#[derive(Debug, Deserialize)]
struct GitHubSearchItem {
    path: String,
    html_url: String,
    repository: GitHubRepository,
    #[serde(default)]
    download_url: Option<String>,
    #[serde(default)]
    text_matches: Option<Vec<TextMatch>>,
}

#[derive(Debug, Deserialize)]
struct TextMatch {
    fragment: String,
}

#[derive(Debug, Deserialize)]
struct GitHubRepository {
    full_name: String,
    #[serde(default = "default_branch")]
    default_branch: String,
}

fn default_branch() -> String {
    "main".to_string()
}

pub struct GitHubProvider {
    tokens: Vec<String>,
    current_token_idx: std::sync::Arc<std::sync::Mutex<usize>>,
    base_url: String,
    rate_limiter: RateLimiter,
}

impl GitHubProvider {
    pub fn new(tokens: Vec<String>, rate_limit_ms: u64) -> Self {
        Self::with_config(tokens, "https://api.github.com".to_string(), rate_limit_ms)
    }

    pub fn with_config(tokens: Vec<String>, base_url: String, rate_limit_ms: u64) -> Self {
        let rate_limiter = RateLimiter::with_delay(Duration::from_millis(rate_limit_ms));

        Self {
            tokens,
            current_token_idx: std::sync::Arc::new(std::sync::Mutex::new(0)),
            base_url,
            rate_limiter,
        }
    }

    fn get_current_token(&self) -> Option<String> {
        if self.tokens.is_empty() {
            return None;
        }
        let idx = *self.current_token_idx.lock().unwrap();
        Some(self.tokens[idx].clone())
    }

    fn rotate_token(&self) -> Option<String> {
        if self.tokens.is_empty() {
            return None;
        }
        let mut idx = self.current_token_idx.lock().unwrap();
        *idx = (*idx + 1) % self.tokens.len();
        let new_token = self.tokens[*idx].clone();
        info!("Rotating to token {} of {}", *idx + 1, self.tokens.len());
        Some(new_token)
    }

    async fn fetch_page(&self, url: &str, token_opt: Option<String>) -> Result<crate::utils::HttpResponse> {
        tokio::task::spawn_blocking({
            let client = HttpClient::new();
            let url = url.to_string();
            move || {
                // Build headers inside the closure
                let mut headers = vec![
                    // Request text matches to get code snippets without downloading files
                    ("Accept", "application/vnd.github.text-match+json".to_string()),
                    ("User-Agent", "curl/7.68.0".to_string()),
                ];

                if let Some(token) = token_opt {
                    headers.push(("Authorization", format!("token {}", token)));
                }

                let header_refs: Vec<(&str, &str)> = headers
                    .iter()
                    .map(|(k, v)| (*k, v.as_str()))
                    .collect();

                client.get(&url, &header_refs)
            }
        })
        .await
        .map_err(|e| KeyHunterError::Unknown(format!("Task join error: {}", e)))?
    }
}

#[async_trait]
impl SearchProvider for GitHubProvider {
    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        info!("Searching GitHub for: {}", query.query);

        let mut all_results = Vec::new();
        let per_page = 100; // GitHub's maximum
        // Calculate max pages needed to reach max_results (use ceiling division)
        let max_pages = ((query.max_results + per_page - 1) / per_page).min(10); // GitHub limits to 1000 results (10 pages)

        // First request to get total count
        let first_url = format!(
            "{}/search/code?q={}&per_page={}&page=1",
            self.base_url,
            urlencoding::encode(&query.query),
            per_page
        );

        self.rate_limiter.wait().await;

        let mut token_opt = self.get_current_token();
        let mut first_result = self.fetch_page(&first_url, token_opt.clone()).await?;

        // If rate limited and we have multiple tokens, try rotating
        if first_result.is_rate_limited() && self.tokens.len() > 1 {
            warn!("Rate limit hit, rotating to next token...");
            token_opt = self.rotate_token();
            first_result = self.fetch_page(&first_url, token_opt.clone()).await?;

            // If still rate limited after trying all tokens, wait
            if first_result.is_rate_limited() {
                warn!("All tokens rate limited, waiting 60 seconds...");
                tokio::time::sleep(Duration::from_secs(60)).await;
                return Err(KeyHunterError::RateLimit(
                    "GitHub API rate limit exceeded for all tokens".to_string(),
                ));
            }
        } else if first_result.is_rate_limited() {
            warn!("GitHub rate limit hit, waiting 60 seconds...");
            tokio::time::sleep(Duration::from_secs(60)).await;
            return Err(KeyHunterError::RateLimit(
                "GitHub API rate limit exceeded".to_string(),
            ));
        }

        if !first_result.is_success() {
            return Err(KeyHunterError::SearchProvider(format!(
                "GitHub API returned {}: {}",
                first_result.status_code,
                first_result.text().unwrap_or_default()
            )));
        }

        let first_response: GitHubSearchResponse = first_result.json()?;
        let total_count = first_response.total_count;
        info!("Found {} total results on GitHub", total_count);

        // Add first page results
        all_results.extend(first_response.items);

        // Calculate how many more pages we need (use ceiling division to get partial pages)
        let total_pages = ((total_count as usize + per_page - 1) / per_page).min(max_pages).max(1);

        if total_pages > 1 && all_results.len() < query.max_results {
            info!("Fetching {} additional pages ({} total)...", total_pages - 1, total_pages);

            for page in 2..=total_pages {
                if all_results.len() >= query.max_results {
                    break;
                }

                let page_url = format!(
                    "{}/search/code?q={}&per_page={}&page={}",
                    self.base_url,
                    urlencoding::encode(&query.query),
                    per_page,
                    page
                );

                // Rate limiting handled by rate_limiter
                self.rate_limiter.wait().await;

                let mut response = match self.fetch_page(&page_url, token_opt.clone()).await {
                    Ok(r) => r,
                    Err(e) => {
                        warn!("Error fetching page {}: {}", page, e);
                        break;
                    }
                };

                // If rate limited and we have multiple tokens, try rotating
                if response.is_rate_limited() && self.tokens.len() > 1 {
                    warn!("Rate limited on page {}, rotating to next token...", page);
                    token_opt = self.rotate_token();
                    response = match self.fetch_page(&page_url, token_opt.clone()).await {
                        Ok(r) => r,
                        Err(e) => {
                            warn!("Error after token rotation: {}", e);
                            break;
                        }
                    };
                }

                // If still rate limited, wait and continue
                if response.is_rate_limited() {
                    warn!("Rate limited on page {}, waiting 60s...", page);
                    tokio::time::sleep(Duration::from_secs(60)).await;
                    continue;
                }

                if !response.is_success() {
                    warn!("Error on page {}: HTTP {}", page, response.status_code);
                    break;
                }

                match response.json::<GitHubSearchResponse>() {
                    Ok(page_response) => {
                        let items_count = page_response.items.len();
                        all_results.extend(page_response.items);
                        debug!("Page {}/{}: +{} results (total: {})",
                            page, total_pages, items_count, all_results.len());

                        if items_count == 0 {
                            break;
                        }
                    }
                    Err(e) => {
                        warn!("Failed to parse page {}: {}", page, e);
                        break;
                    }
                }
            }
        }

        info!("Fetched {} results total", all_results.len());

        // Convert to SearchResult
        let results: Vec<SearchResult> = all_results
            .into_iter()
            .take(query.max_results)
            .map(|item| {
                let download_url = item.download_url.unwrap_or_else(|| {
                    format!(
                        "https://raw.githubusercontent.com/{}/{}/{}",
                        item.repository.full_name,
                        item.repository.default_branch,
                        item.path
                    )
                });

                // Extract text match fragments
                let text_matches = item.text_matches.map(|matches| {
                    matches.into_iter().map(|m| m.fragment).collect()
                });

                SearchResult {
                    repository: item.repository.full_name.clone(),
                    file_path: item.path,
                    file_url: item.html_url,
                    download_url,
                    default_branch: Some(item.repository.default_branch),
                    text_matches,
                }
            })
            .collect();

        Ok(results)
    }

    fn name(&self) -> &str {
        "github"
    }

    async fn get_file_content(&self, result: &SearchResult) -> Result<String> {
        debug!("Downloading file: {}", result.download_url);

        // Wait for rate limiter
        self.rate_limiter.wait().await;

        // Perform request in blocking context
        let response = tokio::task::spawn_blocking({
            let client = HttpClient::new();
            let url = result.download_url.clone();
            move || client.get(&url, &[])
        })
        .await
        .map_err(|e| KeyHunterError::Unknown(format!("Task join error: {}", e)))??;

        if response.is_not_found() {
            return Err(KeyHunterError::NotFound(format!(
                "File not found (likely deleted): {}",
                result.file_path
            )));
        }

        if !response.is_success() {
            return Err(KeyHunterError::Http(format!(
                "Failed to download file: HTTP {}",
                response.status_code
            )));
        }

        response.text()
    }

    fn max_results_per_query(&self) -> usize {
        100
    }
}

// URL encoding utility (simple implementation)
mod urlencoding {
    pub fn encode(s: &str) -> String {
        s.chars()
            .map(|c| match c {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
                ' ' => "+".to_string(),
                _ => format!("%{:02X}", c as u8),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_provider_creation() {
        let provider = GitHubProvider::new(vec![], 2000);
        assert_eq!(provider.name(), "github");
    }

    #[test]
    fn test_github_provider_with_token() {
        let provider = GitHubProvider::new(vec!["ghp_test123".to_string()], 2000);
        assert_eq!(provider.name(), "github");
    }

    #[test]
    fn test_url_encoding() {
        assert_eq!(urlencoding::encode("hello world"), "hello+world");
        assert_eq!(urlencoding::encode("foo@bar"), "foo%40bar");
    }
}
