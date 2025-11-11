use crate::core::error::Result;
use crate::core::results::ValidationResult;
use crate::core::traits::KeyValidator;
use crate::utils::HttpClient;
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct XAIKeyResponse {
    _redacted_api_key: Option<String>,
    user_id: Option<String>,
    team_id: Option<String>,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct XAIErrorResponse {
    error: Option<String>,
    _code: Option<String>,
}

pub struct XAIValidator {
    rate_limit_ms: u64,
}

impl XAIValidator {
    pub fn new(rate_limit_ms: u64) -> Self {
        Self { rate_limit_ms }
    }
}

impl Default for XAIValidator {
    fn default() -> Self {
        Self::new(1000)
    }
}

#[async_trait]
impl KeyValidator for XAIValidator {
    async fn validate(&self, key: &str) -> Result<ValidationResult> {
        // Use the api-key endpoint to validate
        let url = "https://api.x.ai/v1/api-key";

        // Perform request in blocking context (curl is sync)
        let result = tokio::task::spawn_blocking({
            let client = HttpClient::new();
            let url = url.to_string();
            let key = key.to_string();
            move || {
                client.get(
                    &url,
                    &[
                        ("Authorization", &format!("Bearer {}", key)),
                        ("Accept", "application/json"),
                    ],
                )
            }
        })
        .await
        .map_err(|e| crate::core::error::KeyHunterError::Unknown(format!("Task join error: {}", e)))?;

        match result {
            Ok(response) => {
                if response.status_code == 200 {
                    // Try to parse the response to get key info
                    match response.json::<XAIKeyResponse>() {
                        Ok(key_response) => {
                            let mut metadata = HashMap::new();

                            if let Some(user_id) = key_response.user_id {
                                metadata.insert(
                                    "user_id".to_string(),
                                    serde_json::Value::String(user_id),
                                );
                            }

                            if let Some(team_id) = key_response.team_id {
                                metadata.insert(
                                    "team_id".to_string(),
                                    serde_json::Value::String(team_id),
                                );
                            }

                            if let Some(name) = key_response.name {
                                metadata.insert(
                                    "key_name".to_string(),
                                    serde_json::Value::String(name),
                                );
                            }

                            Ok(ValidationResult::valid("xai".to_string(), metadata))
                        }
                        Err(_) => {
                            // Invalid response format but 200 status - still valid
                            let mut metadata = HashMap::new();
                            metadata.insert(
                                "note".to_string(),
                                serde_json::Value::String("Valid key (200 OK)".to_string()),
                            );
                            Ok(ValidationResult::valid("xai".to_string(), metadata))
                        }
                    }
                } else if response.status_code == 400 || response.status_code == 401 {
                    // Invalid API key
                    match response.json::<XAIErrorResponse>() {
                        Ok(error_resp) => {
                            let error_msg = error_resp.error.unwrap_or_else(|| "Invalid API key".to_string());
                            Ok(ValidationResult::invalid(
                                "xai".to_string(),
                                error_msg,
                            ))
                        }
                        Err(_) => {
                            Ok(ValidationResult::invalid(
                                "xai".to_string(),
                                "Invalid API key".to_string(),
                            ))
                        }
                    }
                } else if response.status_code == 403 {
                    // Forbidden - could mean invalid key or disabled API
                    Ok(ValidationResult::invalid(
                        "xai".to_string(),
                        "Forbidden - key may be invalid or API not enabled".to_string(),
                    ))
                } else if response.status_code == 429 {
                    // Rate limited - return error, don't mark key as invalid
                    Err(crate::core::error::KeyHunterError::RateLimit(
                        "xAI API rate limit exceeded".to_string()
                    ))
                } else if response.status_code >= 500 {
                    // Server error - return error, don't mark key as invalid
                    Err(crate::core::error::KeyHunterError::ValidationFailed(
                        format!("xAI API server error: HTTP {}", response.status_code)
                    ))
                } else {
                    // Other client error
                    Err(crate::core::error::KeyHunterError::ValidationFailed(
                        format!("xAI API returned HTTP {}", response.status_code)
                    ))
                }
            }
            Err(e) => {
                // Network or curl error - DON'T mark key as invalid
                Err(crate::core::error::KeyHunterError::Http(
                    format!("Network error validating xAI key: {}", e)
                ))
            }
        }
    }

    fn key_type(&self) -> &str {
        "xai"
    }

    fn rate_limit(&self) -> Duration {
        Duration::from_millis(self.rate_limit_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xai_validator_creation() {
        let validator = XAIValidator::default();
        assert_eq!(validator.key_type(), "xai");
    }
}
