use crate::core::error::Result;
use crate::core::results::ValidationResult;
use crate::core::traits::KeyValidator;
use crate::utils::HttpClient;
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct ClaudeModelsResponse {
    data: Option<Vec<ClaudeModel>>,
}

#[derive(Debug, Deserialize)]
struct ClaudeModel {
    id: String,
    display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ClaudeErrorResponse {
    error: Option<ClaudeError>,
}

#[derive(Debug, Deserialize)]
struct ClaudeError {
    #[serde(rename = "type")]
    _error_type: String,
    message: String,
}

pub struct ClaudeValidator {
    rate_limit_ms: u64,
}

impl ClaudeValidator {
    pub fn new(rate_limit_ms: u64) -> Self {
        Self { rate_limit_ms }
    }
}

impl Default for ClaudeValidator {
    fn default() -> Self {
        Self::new(2000)
    }
}

#[async_trait]
impl KeyValidator for ClaudeValidator {
    async fn validate(&self, key: &str) -> Result<ValidationResult> {
        // Use the models endpoint - free and doesn't consume tokens
        let url = "https://api.anthropic.com/v1/models";

        // Perform request in blocking context (curl is sync)
        let result = tokio::task::spawn_blocking({
            let client = HttpClient::new();
            let key = key.to_string();
            let version = "2023-06-01".to_string();
            move || {
                client.get(
                    url,
                    &[
                        ("x-api-key", &key),
                        ("anthropic-version", &version),
                    ],
                )
            }
        })
        .await
        .map_err(|e| crate::core::error::KeyHunterError::Unknown(format!("Task join error: {}", e)))?;

        match result {
            Ok(response) => {
                if response.status_code == 200 {
                    // Try to parse the response to get model info
                    match response.json::<ClaudeModelsResponse>() {
                        Ok(models_response) => {
                            let mut metadata = HashMap::new();

                            if let Some(models) = models_response.data {
                                let model_count = models.len();
                                metadata.insert(
                                    "model_count".to_string(),
                                    serde_json::Value::Number(model_count.into()),
                                );

                                // List a few model names
                                let model_names: Vec<String> = models
                                    .iter()
                                    .take(3)
                                    .map(|m| m.display_name.clone().unwrap_or_else(|| m.id.clone()))
                                    .collect();

                                if !model_names.is_empty() {
                                    metadata.insert(
                                        "sample_models".to_string(),
                                        serde_json::Value::String(model_names.join(", ")),
                                    );
                                }
                            }

                            Ok(ValidationResult::valid("claude".to_string(), metadata))
                        }
                        Err(_) => {
                            // Invalid response format but 200 status - still valid
                            let mut metadata = HashMap::new();
                            metadata.insert(
                                "note".to_string(),
                                serde_json::Value::String("Valid key (200 OK)".to_string()),
                            );
                            Ok(ValidationResult::valid("claude".to_string(), metadata))
                        }
                    }
                } else if response.status_code == 401 {
                    // Parse error response for details
                    match response.json::<ClaudeErrorResponse>() {
                        Ok(error_resp) => {
                            if let Some(error) = error_resp.error {
                                Ok(ValidationResult::invalid(
                                    "claude".to_string(),
                                    error.message,
                                ))
                            } else {
                                Ok(ValidationResult::invalid(
                                    "claude".to_string(),
                                    "Unauthorized - invalid API key".to_string(),
                                ))
                            }
                        }
                        Err(_) => {
                            Ok(ValidationResult::invalid(
                                "claude".to_string(),
                                "Unauthorized - invalid API key".to_string(),
                            ))
                        }
                    }
                } else if response.status_code == 429 {
                    // Rate limited - return error, don't mark key as invalid
                    Err(crate::core::error::KeyHunterError::RateLimit(
                        "Claude API rate limit exceeded".to_string()
                    ))
                } else if response.status_code >= 500 {
                    // Server error - return error, don't mark key as invalid
                    Err(crate::core::error::KeyHunterError::ValidationFailed(
                        format!("Claude API server error: HTTP {}", response.status_code)
                    ))
                } else {
                    // Other client error
                    Err(crate::core::error::KeyHunterError::ValidationFailed(
                        format!("Claude API returned HTTP {}", response.status_code)
                    ))
                }
            }
            Err(e) => {
                // Network or curl error - DON'T mark key as invalid
                Err(crate::core::error::KeyHunterError::Http(
                    format!("Network error validating Claude key: {}", e)
                ))
            }
        }
    }

    fn key_type(&self) -> &str {
        "claude"
    }

    fn rate_limit(&self) -> Duration {
        Duration::from_millis(self.rate_limit_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_validator_creation() {
        let validator = ClaudeValidator::default();
        assert_eq!(validator.key_type(), "claude");
    }
}
