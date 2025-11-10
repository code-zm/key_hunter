use crate::core::error::Result;
use crate::core::results::ValidationResult;
use crate::core::traits::KeyValidator;
use crate::utils::HttpClient;
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct SlackAuthResponse {
    ok: bool,
    team: Option<String>,
    user: Option<String>,
    team_id: Option<String>,
}

pub struct SlackValidator;

impl SlackValidator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SlackValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl KeyValidator for SlackValidator {
    async fn validate(&self, key: &str) -> Result<ValidationResult> {
        let url = "https://slack.com/api/auth.test";

        // Perform request in blocking context (curl is sync)
        let result = tokio::task::spawn_blocking({
            let client = HttpClient::new();
            let auth_header = format!("Bearer {}", key);
            move || {
                client.get(
                    url,
                    &[
                        ("Authorization", &auth_header),
                        ("Content-Type", "application/x-www-form-urlencoded"),
                    ],
                )
            }
        })
        .await
        .map_err(|e| crate::core::error::KeyHunterError::Unknown(format!("Task join error: {}", e)))?;

        match result {
            Ok(response) => {
                if response.status_code == 200 {
                    // Slack always returns 200, check the 'ok' field in response
                    match response.json::<SlackAuthResponse>() {
                        Ok(auth_resp) => {
                            if auth_resp.ok {
                                let mut metadata = HashMap::new();

                                if let Some(team) = auth_resp.team {
                                    metadata.insert(
                                        "team".to_string(),
                                        serde_json::Value::String(team),
                                    );
                                }

                                if let Some(user) = auth_resp.user {
                                    metadata.insert(
                                        "user".to_string(),
                                        serde_json::Value::String(user),
                                    );
                                }

                                if let Some(team_id) = auth_resp.team_id {
                                    metadata.insert(
                                        "team_id".to_string(),
                                        serde_json::Value::String(team_id),
                                    );
                                }

                                Ok(ValidationResult::valid("slack".to_string(), metadata))
                            } else {
                                // ok: false means invalid token
                                Ok(ValidationResult::invalid(
                                    "slack".to_string(),
                                    "Invalid token".to_string(),
                                ))
                            }
                        }
                        Err(_) => {
                            // Failed to parse response
                            Err(crate::core::error::KeyHunterError::ValidationFailed(
                                "Failed to parse Slack API response".to_string()
                            ))
                        }
                    }
                } else if response.status_code == 429 {
                    // Rate limited - return error, don't mark key as invalid
                    Err(crate::core::error::KeyHunterError::RateLimit(
                        "Slack API rate limit exceeded".to_string()
                    ))
                } else if response.status_code >= 500 {
                    // Server error - return error, don't mark key as invalid
                    Err(crate::core::error::KeyHunterError::ValidationFailed(
                        format!("Slack API server error: HTTP {}", response.status_code)
                    ))
                } else {
                    // Other client error
                    Err(crate::core::error::KeyHunterError::ValidationFailed(
                        format!("Slack API returned HTTP {}", response.status_code)
                    ))
                }
            }
            Err(e) => {
                // Network or curl error - DON'T mark key as invalid
                Err(crate::core::error::KeyHunterError::Http(
                    format!("Network error validating Slack token: {}", e)
                ))
            }
        }
    }

    fn key_type(&self) -> &str {
        "slack"
    }

    fn rate_limit(&self) -> Duration {
        // Slack auth.test has generous rate limits - 1 second between requests
        Duration::from_millis(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slack_validator_creation() {
        let validator = SlackValidator::new();
        assert_eq!(validator.key_type(), "slack");
    }
}
