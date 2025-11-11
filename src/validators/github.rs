use crate::core::error::Result;
use crate::core::results::ValidationResult;
use crate::core::traits::KeyValidator;
use crate::utils::HttpClient;
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct GitHubUser {
    login: String,
    id: i64,
    #[serde(rename = "type")]
    user_type: Option<String>,
}

pub struct GitHubValidator {
    rate_limit_ms: u64,
}

impl GitHubValidator {
    pub fn new(rate_limit_ms: u64) -> Self {
        Self { rate_limit_ms }
    }
}

impl Default for GitHubValidator {
    fn default() -> Self {
        Self::new(2000)
    }
}

#[async_trait]
impl KeyValidator for GitHubValidator {
    async fn validate(&self, key: &str) -> Result<ValidationResult> {
        let url = "https://api.github.com/user";

        // Perform request in blocking context (curl is sync)
        let result = tokio::task::spawn_blocking({
            let client = HttpClient::new();
            let auth_header = format!("Bearer {}", key);
            let user_agent = "KeyHunter/1.0".to_string();
            move || {
                client.get(
                    url,
                    &[
                        ("Authorization", &auth_header),
                        ("User-Agent", &user_agent),
                        ("Accept", "application/vnd.github+json"),
                    ],
                )
            }
        })
        .await
        .map_err(|e| crate::core::error::KeyHunterError::Unknown(format!("Task join error: {}", e)))?;

        match result {
            Ok(response) => {
                if response.status_code == 200 {
                    // Try to parse the response
                    match response.json::<GitHubUser>() {
                        Ok(user) => {
                            let mut metadata = HashMap::new();

                            metadata.insert(
                                "login".to_string(),
                                serde_json::Value::String(user.login),
                            );

                            metadata.insert(
                                "user_id".to_string(),
                                serde_json::Value::Number(user.id.into()),
                            );

                            if let Some(user_type) = user.user_type {
                                metadata.insert(
                                    "type".to_string(),
                                    serde_json::Value::String(user_type),
                                );
                            }

                            Ok(ValidationResult::valid("github".to_string(), metadata))
                        }
                        Err(_) => {
                            // Invalid response format but 200 status - still valid
                            let mut metadata = HashMap::new();
                            metadata.insert(
                                "note".to_string(),
                                serde_json::Value::String("Valid token (200 OK)".to_string()),
                            );
                            Ok(ValidationResult::valid("github".to_string(), metadata))
                        }
                    }
                } else if response.status_code == 401 {
                    // ONLY 401 means invalid token
                    Ok(ValidationResult::invalid(
                        "github".to_string(),
                        "Unauthorized - token is invalid or revoked".to_string(),
                    ))
                } else if response.status_code == 403 {
                    // Could be rate limit or permissions issue
                    // Check if it's rate limit by looking at response
                    let body_text = response.text().unwrap_or_default();
                    if body_text.contains("rate limit") || body_text.contains("API rate limit") {
                        Err(crate::core::error::KeyHunterError::RateLimit(
                            "GitHub API rate limit exceeded".to_string()
                        ))
                    } else {
                        // Forbidden but not rate limit - could be token with insufficient permissions
                        // This doesn't mean the token is invalid, just lacks permissions
                        Err(crate::core::error::KeyHunterError::ValidationFailed(
                            "Token valid but lacks required permissions".to_string()
                        ))
                    }
                } else if response.status_code == 429 {
                    // Rate limited - return error, don't mark token as invalid
                    Err(crate::core::error::KeyHunterError::RateLimit(
                        "GitHub API rate limit exceeded".to_string()
                    ))
                } else if response.status_code >= 500 {
                    // Server error - return error, don't mark token as invalid
                    Err(crate::core::error::KeyHunterError::ValidationFailed(
                        format!("GitHub API server error: HTTP {}", response.status_code)
                    ))
                } else {
                    // Other client error
                    Err(crate::core::error::KeyHunterError::ValidationFailed(
                        format!("GitHub API returned HTTP {}", response.status_code)
                    ))
                }
            }
            Err(e) => {
                // Network or curl error - DON'T mark token as invalid
                Err(crate::core::error::KeyHunterError::Http(
                    format!("Network error validating GitHub token: {}", e)
                ))
            }
        }
    }

    fn key_type(&self) -> &str {
        "github"
    }

    fn rate_limit(&self) -> Duration {
        Duration::from_millis(self.rate_limit_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_validator_creation() {
        let validator = GitHubValidator::default();
        assert_eq!(validator.key_type(), "github");
    }
}
