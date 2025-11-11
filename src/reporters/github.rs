use crate::core::{KeyHunterError, Result, ValidatedKey};
use crate::reporters::get_reporter;
use crate::utils::HttpClient;
use indicatif::ProgressBar;
use serde_json::json;
use std::collections::{HashMap, HashSet};
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

    /// Check if an issue with our title pattern already exists in the repository
    async fn check_issue_exists(&self, repo: &str, expected_title: &str) -> Result<bool> {
        let url = format!("https://api.github.com/repos/{}/issues?state=all&per_page=100", repo);

        // Perform request in blocking context (curl is sync)
        let result = tokio::task::spawn_blocking({
            let client = HttpClient::new();
            let token = self.github_token.clone();
            let url = url.clone();
            move || {
                let headers = &[
                    ("Authorization", &*format!("Bearer {}", token)),
                    ("Accept", "application/vnd.github.v3+json"),
                    ("User-Agent", "key-hunter"),
                ];
                client.get(&url, headers)
            }
        })
        .await
        .map_err(|e| KeyHunterError::Unknown(format!("Task join error: {}", e)))?;

        let response = match result {
            Ok(resp) => resp,
            Err(_) => return Ok(false), // Network error - don't block issue creation
        };

        match response.status_code {
            200 => {
                // Parse the response to check if any issue matches our title exactly
                if let Ok(issues) = response.json::<serde_json::Value>() {
                    if let Some(issues_array) = issues.as_array() {
                        for issue in issues_array {
                            if let Some(title) = issue["title"].as_str() {
                                // Check if the title matches our expected title
                                if title == expected_title {
                                    info!("Found existing issue in {}: {}", repo, title);
                                    return Ok(true);
                                }
                            }
                        }
                    }
                }
                Ok(false)
            }
            404 => {
                // Repository not found - let create_issue handle this
                Ok(false)
            }
            403 => {
                // Permission denied - let create_issue handle this
                Ok(false)
            }
            _ => {
                // Other errors - log but don't block issue creation
                warn!("Failed to check existing issues for {}: HTTP {}", repo, response.status_code);
                Ok(false)
            }
        }
    }

    /// Format a title for multiple keys
    fn format_multi_key_title(&self, key_type: &str, count: usize) -> String {
        let service_name = crate::reporters::ServiceConfig::get(key_type).service_name;
        format!("[Security] {} Exposed {} API keys", count, service_name)
    }

    /// Format a body for multiple keys
    fn format_multi_key_body(&self, validated_keys: &[ValidatedKey]) -> Result<String> {
        use crate::reporters::template::TemplateRenderer;

        let first_key = &validated_keys[0];
        let service_config = crate::reporters::ServiceConfig::get(&first_key.detected.key_type);
        let count = validated_keys.len();

        let template = match TemplateRenderer::load("issue") {
            Ok(t) => t,
            Err(_) => {
                let files: Vec<String> = validated_keys.iter()
                    .map(|k| format!("- {} at {}", k.detected.file_path, k.detected.file_url))
                    .collect();
                return Ok(format!(
                    "Multiple exposed {} API keys found:\n\n{}\n\nPlease revoke all keys immediately.",
                    first_key.detected.key_type.to_uppercase(),
                    files.join("\n")
                ));
            }
        };

        let mut vars = HashMap::new();

        // Service info
        vars.insert("service_name".to_string(), service_config.service_name.clone());
        vars.insert("revoke_url".to_string(), service_config.revoke_url.clone());
        vars.insert("additional_actions".to_string(), service_config.additional_actions.clone());
        vars.insert("best_practices".to_string(), service_config.best_practices.clone());
        vars.insert("resources".to_string(), service_config.resources.clone());

        // Plural/singular variables
        vars.insert("key_count".to_string(), count.to_string());
        vars.insert("key_count_plural".to_string(), if count > 1 { "s" } else { "" }.to_string());
        vars.insert("key_count_plural_upper".to_string(), if count > 1 { "S" } else { "" }.to_string());
        vars.insert("key_count_verb".to_string(), if count > 1 { "are" } else { "is" }.to_string());
        vars.insert("key_count_verb_past".to_string(), if count > 1 { "were" } else { "was" }.to_string());
        vars.insert("key_count_these".to_string(), if count > 1 { "These" } else { "This" }.to_string());
        vars.insert("key_count_these_upper".to_string(), if count > 1 { "THESE" } else { "THIS" }.to_string());
        vars.insert("key_count_the".to_string(), if count > 1 { "these" } else { "the" }.to_string());

        // Build keys details section
        let keys_details: Vec<String> = validated_keys.iter()
            .enumerate()
            .map(|(i, k)| {
                // Create partial key preview (first 8 chars + ... + last 4 chars)
                let key_preview = if k.detected.key.len() > 12 {
                    format!("{}...{}", &k.detected.key[..8], &k.detected.key[k.detected.key.len()-4..])
                } else {
                    format!("{}...", &k.detected.key[..k.detected.key.len().min(8)])
                };

                format!(
                    "**Key {}:**\n- **File**: `{}`\n- **Line Number**: {}\n- **File URL**: {}\n- **Key Preview**: `{}` (truncated for security)",
                    i + 1,
                    k.detected.file_path,
                    k.detected.line_number.map(|n| n.to_string()).unwrap_or_else(|| "N/A".to_string()),
                    k.detected.file_url,
                    key_preview
                )
            })
            .collect();
        vars.insert("keys_details".to_string(), keys_details.join("\n\n"));

        // Build metadata section
        let metadata_parts: Vec<String> = validated_keys.iter()
            .enumerate()
            .map(|(i, k)| {
                if k.validation.metadata.is_empty() {
                    format!("**Key {}**: Validated successfully", i + 1)
                } else {
                    let meta_items: Vec<String> = k.validation.metadata.iter()
                        .map(|(key, value)| format!("  - **{}**: {}", key, value))
                        .collect();
                    format!("**Key {}**:\n{}", i + 1, meta_items.join("\n"))
                }
            })
            .collect();
        vars.insert("metadata_section".to_string(), metadata_parts.join("\n\n"));

        // Build file cleanup commands
        let file_paths: Vec<&str> = validated_keys.iter()
            .map(|k| k.detected.file_path.as_str())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        let cleanup_commands: Vec<String> = file_paths.iter()
            .map(|path| format!("git filter-repo --path {} --invert-paths", path))
            .collect();
        vars.insert("file_cleanup_commands".to_string(), cleanup_commands.join("\n"));

        // Timestamp
        vars.insert("timestamp".to_string(), chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string());

        Ok(template.render(&vars))
    }

    /// Create an issue for one or more exposed keys in a repository
    pub async fn create_issue(&self, repo: &str, validated_keys: &[ValidatedKey]) -> Result<String> {
        if validated_keys.is_empty() {
            return Err(KeyHunterError::ValidationFailed("No keys provided".to_string()));
        }

        let first_key = &validated_keys[0];
        let reporter = get_reporter(&first_key.detected.key_type)
            .ok_or_else(|| KeyHunterError::Unknown(format!("No reporter for key type: {}", first_key.detected.key_type)))?;

        // For multiple keys, generate a combined title and body
        let (title, body) = if validated_keys.len() == 1 {
            let title = reporter.format_issue_title(&first_key.detected);
            let body = reporter.format_issue_body(&first_key.detected, &first_key.validation);
            (title, body)
        } else {
            // Multiple keys - create combined issue
            let title = self.format_multi_key_title(&first_key.detected.key_type, validated_keys.len());
            let body = self.format_multi_key_body(validated_keys)?;
            (title, body)
        };

        // Check if an issue already exists (skip in dry-run mode)
        if !self.dry_run {
            if self.check_issue_exists(repo, &title).await? {
                info!("Issue already exists in {}, skipping", repo);
                return Err(KeyHunterError::ValidationFailed(
                    format!("Issue already exists in repository")
                ));
            }
        }

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
            "body": body
        });

        let headers = &[
            ("Authorization", &*format!("Bearer {}", self.github_token)),
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

    /// Create issues for multiple validated keys, grouping by repository
    pub async fn create_issues_bulk(
        &self,
        validated_keys: &[ValidatedKey],
        progress_bar: Option<&ProgressBar>,
    ) -> Result<IssueCreationStats> {
        let mut stats = IssueCreationStats::default();
        stats.total = validated_keys.len();

        // Group keys by repository
        let mut keys_by_repo: HashMap<String, Vec<ValidatedKey>> = HashMap::new();
        for key in validated_keys {
            keys_by_repo.entry(key.detected.repository.clone())
                .or_insert_with(Vec::new)
                .push(key.clone());
        }

        info!("Grouped {} keys into {} repositories", validated_keys.len(), keys_by_repo.len());

        // Update progress bar to show repository count
        if let Some(pb) = progress_bar {
            pb.set_length(keys_by_repo.len() as u64);
            pb.set_position(0);
        }

        // Create one issue per repository
        for (repo, keys) in keys_by_repo {
            // Update progress message
            if let Some(pb) = progress_bar {
                let key_word = if keys.len() > 1 { "keys" } else { "key" };
                pb.set_message(format!("Processing {} ({} {})", repo, keys.len(), key_word));
            }

            match self.create_issue(&repo, &keys).await {
                Ok(url) => {
                    stats.success += 1;
                    stats.issue_urls.push(url);
                }
                Err(e) => {
                    // Check if this is an "already exists" error
                    let error_msg = e.to_string();
                    if error_msg.contains("Issue already exists") {
                        info!("Issue already exists in {}, skipping", repo);
                        stats.skipped += keys.len();
                    } else {
                        stats.failed += keys.len();
                        stats.errors.push(format!("{}: {}", repo, e));
                    }
                }
            }

            // Increment progress
            if let Some(pb) = progress_bar {
                pb.inc(1);
            }

            // Rate limit: wait 1 second between issue creation
            if !self.dry_run {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }

        if let Some(pb) = progress_bar {
            pb.finish_with_message("Issue creation complete");
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
