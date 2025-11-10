use crate::core::error::Result;
use crate::core::results::ValidationResult;
use crate::core::traits::KeyValidator;
use crate::utils::HttpClient;
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct OpenAIModelsResponse {
    data: Vec<OpenAIModel>,
}

#[derive(Debug, Deserialize)]
struct OpenAIModel {
    id: String,
}

pub struct OpenAIValidator;

impl OpenAIValidator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for OpenAIValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl KeyValidator for OpenAIValidator {
    async fn validate(&self, key: &str) -> Result<ValidationResult> {
        let url = "https://api.openai.com/v1/models";

        // Perform request in blocking context (curl is sync)
        let result = tokio::task::spawn_blocking({
            let client = HttpClient::new();
            let key = key.to_string();
            move || {
                client.get(
                    url,
                    &[
                        ("Authorization", &format!("Bearer {}", key)),
                        ("Content-Type", "application/json"),
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
                    match response.json::<OpenAIModelsResponse>() {
                        Ok(models_response) => {
                            let mut metadata = HashMap::new();

                            // Count models
                            let model_count = models_response.data.len();
                            metadata.insert(
                                "model_count".to_string(),
                                serde_json::Value::Number(model_count.into()),
                            );

                            // List a few model names
                            let model_names: Vec<String> = models_response
                                .data
                                .iter()
                                .take(3)
                                .map(|m| m.id.clone())
                                .collect();

                            if !model_names.is_empty() {
                                metadata.insert(
                                    "sample_models".to_string(),
                                    serde_json::Value::String(model_names.join(", ")),
                                );
                            }

                            Ok(ValidationResult::valid("openai".to_string(), metadata))
                        }
                        Err(_) => {
                            // Invalid response format but 200 status - still valid
                            let mut metadata = HashMap::new();
                            metadata.insert(
                                "note".to_string(),
                                serde_json::Value::String("Valid key (200 OK)".to_string()),
                            );
                            Ok(ValidationResult::valid("openai".to_string(), metadata))
                        }
                    }
                } else if response.status_code == 401 {
                    // ONLY 401 means invalid key
                    Ok(ValidationResult::invalid(
                        "openai".to_string(),
                        "Unauthorized - key is invalid or revoked".to_string(),
                    ))
                } else if response.status_code == 429 {
                    // Rate limited - return error, don't mark key as invalid
                    Err(crate::core::error::KeyHunterError::RateLimit(
                        "OpenAI API rate limit exceeded".to_string()
                    ))
                } else if response.status_code >= 500 {
                    // Server error - return error, don't mark key as invalid
                    Err(crate::core::error::KeyHunterError::ValidationFailed(
                        format!("OpenAI API server error: HTTP {}", response.status_code)
                    ))
                } else {
                    // Other client error - could be invalid but might be temporary
                    Err(crate::core::error::KeyHunterError::ValidationFailed(
                        format!("OpenAI API returned HTTP {}", response.status_code)
                    ))
                }
            }
            Err(e) => {
                // Network or curl error - DON'T mark key as invalid
                Err(crate::core::error::KeyHunterError::Http(
                    format!("Network error validating OpenAI key: {}", e)
                ))
            }
        }
    }

    fn key_type(&self) -> &str {
        "openai"
    }

    fn rate_limit(&self) -> Duration {
        // OpenAI has rate limits - 2 seconds between validation requests
        // This helps avoid rate limit errors when validating many keys
        Duration::from_millis(2000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_validator_creation() {
        let validator = OpenAIValidator::new();
        assert_eq!(validator.key_type(), "openai");
    }

    #[tokio::test]
    async fn test_validate_invalid_key() {
        let validator = OpenAIValidator::new();
        let result = validator.validate("sk-invalidkey123456789012345678901234567890123456").await;

        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(!validation.valid);
    }
}
