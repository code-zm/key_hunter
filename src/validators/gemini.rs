use crate::core::error::Result;
use crate::core::results::ValidationResult;
use crate::core::traits::KeyValidator;
use crate::utils::HttpClient;
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct GeminiModelsResponse {
    models: Option<Vec<GeminiModel>>,
}

#[derive(Debug, Deserialize)]
struct GeminiModel {
    name: String,
}

pub struct GeminiValidator;

impl GeminiValidator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GeminiValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl KeyValidator for GeminiValidator {
    async fn validate(&self, key: &str) -> Result<ValidationResult> {
        // Use the models endpoint to validate the key
        let url = format!("https://generativelanguage.googleapis.com/v1beta/models?key={}", key);

        // Perform request in blocking context (curl is sync)
        let result = tokio::task::spawn_blocking({
            let client = HttpClient::new();
            let url = url.clone();
            move || client.get(&url, &[("Accept", "application/json")])
        })
        .await
        .map_err(|e| crate::core::error::KeyHunterError::Unknown(format!("Task join error: {}", e)))?;

        match result {
            Ok(response) => {
                if response.status_code == 200 {
                    // Try to parse the response to get model info
                    match response.json::<GeminiModelsResponse>() {
                        Ok(models_response) => {
                            let mut metadata = HashMap::new();

                            if let Some(models) = models_response.models {
                                let model_count = models.len();
                                metadata.insert(
                                    "model_count".to_string(),
                                    serde_json::Value::Number(model_count.into()),
                                );

                                // List a few model names
                                let model_names: Vec<String> = models
                                    .iter()
                                    .take(3)
                                    .map(|m| m.name.clone())
                                    .collect();

                                if !model_names.is_empty() {
                                    metadata.insert(
                                        "sample_models".to_string(),
                                        serde_json::Value::String(model_names.join(", ")),
                                    );
                                }
                            }

                            Ok(ValidationResult::valid("gemini".to_string(), metadata))
                        }
                        Err(_) => {
                            // Invalid response format but 200 status - still valid
                            let mut metadata = HashMap::new();
                            metadata.insert(
                                "note".to_string(),
                                serde_json::Value::String("Valid key (200 OK)".to_string()),
                            );
                            Ok(ValidationResult::valid("gemini".to_string(), metadata))
                        }
                    }
                } else if response.status_code == 400 {
                    // Gemini returns 400 for invalid API keys
                    Ok(ValidationResult::invalid(
                        "gemini".to_string(),
                        "Invalid API key".to_string(),
                    ))
                } else if response.status_code == 403 {
                    // Forbidden - could mean invalid key or disabled API
                    Ok(ValidationResult::invalid(
                        "gemini".to_string(),
                        "Forbidden - key may be invalid or API not enabled".to_string(),
                    ))
                } else if response.status_code == 429 {
                    // Rate limited - return error, don't mark key as invalid
                    Err(crate::core::error::KeyHunterError::RateLimit(
                        "Gemini API rate limit exceeded".to_string()
                    ))
                } else if response.status_code >= 500 {
                    // Server error - return error, don't mark key as invalid
                    Err(crate::core::error::KeyHunterError::ValidationFailed(
                        format!("Gemini API server error: HTTP {}", response.status_code)
                    ))
                } else {
                    // Other client error
                    Err(crate::core::error::KeyHunterError::ValidationFailed(
                        format!("Gemini API returned HTTP {}", response.status_code)
                    ))
                }
            }
            Err(e) => {
                // Network or curl error - DON'T mark key as invalid
                Err(crate::core::error::KeyHunterError::Http(
                    format!("Network error validating Gemini key: {}", e)
                ))
            }
        }
    }

    fn key_type(&self) -> &str {
        "gemini"
    }

    fn rate_limit(&self) -> Duration {
        // Gemini has rate limits - 2 seconds between validation requests
        Duration::from_millis(2000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemini_validator_creation() {
        let validator = GeminiValidator::new();
        assert_eq!(validator.key_type(), "gemini");
    }
}
