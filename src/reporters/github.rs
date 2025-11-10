use crate::core::{KeyHunterError, Result, ValidatedKey};
use crate::reporters::get_reporter;
use crate::utils::HttpClient;
use serde_json::json;
use std::collections::HashSet;
use tracing::{info, warn};

/// Client for creating GitHub issues
pub struct GitHubIssueClient {
    http_client: HttpClient,
    github_token: String,
    dry_run: bool,
}

impl GitHubIssueClient {
    pub fn new(github_token: String, dry_run: bool) -> Self {
        Self {
            http_client: HttpClient::new(),
            github_token,
            dry_run,
        }
    }

    /// Create an issue for an exposed key
    pub async fn create_issue(&self, repo: &str, validated_key: &ValidatedKey) -> Result<String> {
        let reporter = get_reporter(&validated_key.detected.key_type)
            .ok_or_else(|| KeyHunterError::Unknown(format!("No reporter for key type: {}", validated_key.detected.key_type)))?;

        let title = reporter.format_issue_title(&validated_key.detected);
        let body = reporter.format_issue_body(&validated_key.detected, &validated_key.validation);

        if self.dry_run {
            println!("\n{}", "=".repeat(80));
            println!("DRY RUN: Would create issue in {}", repo);
            println!("Title: {}", title);
            println!("{}", "=".repeat(80));
            println!("{}", body);
            println!("{}", "=".repeat(80));
            return Ok(format!("DRY RUN: {}", repo));
        }

        let url = format!("https://api.github.com/repos/{}/issues", repo);
        let payload = json!({
            "title": title,
            "body": body,
            "labels": ["security", "exposed-credentials", "urgent"]
        });

        let headers = &[
            ("Authorization", &*format!("token {}", self.github_token)),
            ("Accept", "application/vnd.github.v3+json"),
            ("Content-Type", "application/json"),
            ("User-Agent", "key-hunter"),
        ];

        let response = self.http_client.post(&url, headers, &payload.to_string())?;

        match response.status_code {
            201 => {
                let json: serde_json::Value = response.json()?;
                let issue_url = json["html_url"].as_str().unwrap_or("unknown");
                info!("Created issue: {}", issue_url);
                Ok(issue_url.to_string())
            }
            410 => {
                warn!("Issues are disabled for {}", repo);
                Err(KeyHunterError::Http(format!("Issues disabled for {}", repo)))
            }
            404 => {
                warn!("Repository {} not found or not accessible", repo);
                Err(KeyHunterError::NotFound(format!("Repository {}", repo)))
            }
            403 => {
                warn!("Permission denied for {} (may be private or token lacks permissions)", repo);
                Err(KeyHunterError::Http(format!("Permission denied for {}", repo)))
            }
            _ => {
                let error_msg = response.text().unwrap_or_else(|_| "Unknown error".to_string());
                Err(KeyHunterError::Http(format!("Failed to create issue ({}): {}", response.status_code, error_msg)))
            }
        }
    }

    /// Create issues for multiple validated keys, deduplicating by repository
    pub async fn create_issues_bulk(&self, validated_keys: &[ValidatedKey]) -> Result<IssueCreationStats> {
        let mut stats = IssueCreationStats::default();
        let mut repos_processed = HashSet::new();

        for validated_key in validated_keys {
            stats.total += 1;

            let repo = &validated_key.detected.repository;

            // Skip if we already processed this repo
            if repos_processed.contains(repo) {
                info!("Skipping duplicate for {}", repo);
                stats.skipped += 1;
                continue;
            }

            repos_processed.insert(repo.clone());

            match self.create_issue(repo, validated_key).await {
                Ok(url) => {
                    stats.success += 1;
                    stats.issue_urls.push(url);
                }
                Err(e) => {
                    stats.failed += 1;
                    stats.errors.push(format!("{}: {}", repo, e));
                }
            }

            // Rate limit: wait 1 second between issue creation
            if !self.dry_run {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }

        Ok(stats)
    }
}

#[derive(Debug, Default)]
pub struct IssueCreationStats {
    pub total: usize,
    pub success: usize,
    pub failed: usize,
    pub skipped: usize,
    pub issue_urls: Vec<String>,
    pub errors: Vec<String>,
}
