use crate::core::error::Result;
use crate::core::results::ValidationResult;
use crate::core::traits::KeyValidator;
use crate::utils::HttpClient;
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct OpenRouterCreditsResponse {
    data: OpenRouterCreditsData,
}

#[derive(Debug, Deserialize)]
struct OpenRouterCreditsData {
    total_credits: f64,
    total_usage: f64,
}

pub struct OpenRouterValidator;

impl OpenRouterValidator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for OpenRouterValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl KeyValidator for OpenRouterValidator {
    async fn validate(&self, key: &str) -> Result<ValidationResult> {
        let url = "https://openrouter.ai/api/v1/credits";

        // Perform request in blocking context (curl is sync)
        let result = tokio::task::spawn_blocking({
            let client = HttpClient::new();
            let key = key.to_string();
            move || {
                client.get(
                    url,
                    &[
                        ("Authorization", &format!("Bearer {}", key)),
                    ],
                )
            }
        })
        .await
        .map_err(|e| crate::core::error::KeyHunterError::Unknown(format!("Task join error: {}", e)))?;

        match result {
            Ok(response) => {
                if response.status_code == 200 {
                    // Try to parse the response to get credits info
                    match response.json::<OpenRouterCreditsResponse>() {
                        Ok(credits_response) => {
                            let mut metadata = HashMap::new();

                            metadata.insert(
                                "total_credits".to_string(),
                                serde_json::Value::Number(
                                    serde_json::Number::from_f64(credits_response.data.total_credits)
                                        .unwrap_or_else(|| serde_json::Number::from(0))
                                ),
                            );

                            metadata.insert(
                                "total_usage".to_string(),
                                serde_json::Value::Number(
                                    serde_json::Number::from_f64(credits_response.data.total_usage)
                                        .unwrap_or_else(|| serde_json::Number::from(0))
                                ),
                            );

                            let remaining = credits_response.data.total_credits - credits_response.data.total_usage;
                            metadata.insert(
                                "remaining_credits".to_string(),
                                serde_json::Value::Number(
                                    serde_json::Number::from_f64(remaining)
                                        .unwrap_or_else(|| serde_json::Number::from(0))
                                ),
                            );

                            Ok(ValidationResult::valid("openrouter".to_string(), metadata))
                        }
                        Err(_) => {
                            // Invalid response format but 200 status - still valid
                            let mut metadata = HashMap::new();
                            metadata.insert(
                                "note".to_string(),
                                serde_json::Value::String("Valid key (200 OK)".to_string()),
                            );
                            Ok(ValidationResult::valid("openrouter".to_string(), metadata))
                        }
                    }
                } else if response.status_code == 401 {
                    // Unauthorized means invalid key
                    Ok(ValidationResult::invalid(
                        "openrouter".to_string(),
                        "Unauthorized - key is invalid or revoked".to_string(),
                    ))
                } else if response.status_code == 403 {
                    // Forbidden - might still be valid but lacks permissions
                    Ok(ValidationResult::invalid(
                        "openrouter".to_string(),
                        "Forbidden - key lacks required permissions".to_string(),
                    ))
                } else if response.status_code == 429 {
                    // Rate limited - return error, don't mark key as invalid
                    Err(crate::core::error::KeyHunterError::RateLimit(
                        "OpenRouter API rate limit exceeded".to_string()
                    ))
                } else if response.status_code >= 500 {
                    // Server error - return error, don't mark key as invalid
                    Err(crate::core::error::KeyHunterError::ValidationFailed(
                        format!("OpenRouter API server error: HTTP {}", response.status_code)
                    ))
                } else {
                    // Other client error - could be invalid but might be temporary
                    Err(crate::core::error::KeyHunterError::ValidationFailed(
                        format!("OpenRouter API returned HTTP {}", response.status_code)
                    ))
                }
            }
            Err(e) => {
                // Network or curl error - DON'T mark key as invalid
                Err(crate::core::error::KeyHunterError::Http(
                    format!("Network error validating OpenRouter key: {}", e)
                ))
            }
        }
    }

    fn key_type(&self) -> &str {
        "openrouter"
    }

    fn rate_limit(&self) -> Duration {
        // Be conservative with rate limits
        Duration::from_millis(2000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openrouter_validator_creation() {
        let validator = OpenRouterValidator::new();
        assert_eq!(validator.key_type(), "openrouter");
    }
}
