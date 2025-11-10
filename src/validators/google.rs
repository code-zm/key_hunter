use crate::core::error::Result;
use crate::core::results::ValidationResult;
use crate::core::traits::KeyValidator;
use crate::utils::HttpClient;
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct GoogleErrorResponse {
    error: Option<GoogleError>,
}

#[derive(Debug, Deserialize)]
struct GoogleError {
    code: i32,
    message: String,
}

pub struct GoogleValidator;

impl GoogleValidator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GoogleValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl KeyValidator for GoogleValidator {
    async fn validate(&self, key: &str) -> Result<ValidationResult> {
        // Use YouTube Data API v3 search endpoint - it's commonly enabled and lightweight
        let url = format!(
            "https://www.googleapis.com/youtube/v3/search?part=snippet&maxResults=1&q=test&key={}",
            key
        );

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
                    // Valid key with YouTube API enabled
                    let mut metadata = HashMap::new();
                    metadata.insert(
                        "api_enabled".to_string(),
                        serde_json::Value::String("youtube_data_v3".to_string()),
                    );
                    Ok(ValidationResult::valid("google".to_string(), metadata))
                } else if response.status_code == 400 {
                    // Parse error response to determine if it's invalid key or disabled API
                    match response.json::<GoogleErrorResponse>() {
                        Ok(error_resp) => {
                            if let Some(error) = error_resp.error {
                                if error.code == 400 && error.message.contains("API key not valid") {
                                    // Invalid API key
                                    Ok(ValidationResult::invalid(
                                        "google".to_string(),
                                        "API key not valid".to_string(),
                                    ))
                                } else if error.code == 403 || error.message.contains("not enabled") {
                                    // API not enabled but key might be valid - try alternate validation
                                    // This isn't conclusive so return error
                                    Err(crate::core::error::KeyHunterError::ValidationFailed(
                                        "YouTube API not enabled - cannot validate key".to_string()
                                    ))
                                } else {
                                    Err(crate::core::error::KeyHunterError::ValidationFailed(
                                        format!("Google API error: {}", error.message)
                                    ))
                                }
                            } else {
                                Err(crate::core::error::KeyHunterError::ValidationFailed(
                                    "Google API returned 400 with unknown error".to_string()
                                ))
                            }
                        }
                        Err(_) => {
                            Err(crate::core::error::KeyHunterError::ValidationFailed(
                                "Failed to parse Google API error response".to_string()
                            ))
                        }
                    }
                } else if response.status_code == 403 {
                    // Could be disabled API or invalid key - check error message
                    match response.json::<GoogleErrorResponse>() {
                        Ok(error_resp) => {
                            if let Some(error) = error_resp.error {
                                if error.message.contains("API key not valid") || error.message.contains("invalid") {
                                    Ok(ValidationResult::invalid(
                                        "google".to_string(),
                                        "Invalid API key".to_string(),
                                    ))
                                } else {
                                    // Likely disabled API, not invalid key
                                    Err(crate::core::error::KeyHunterError::ValidationFailed(
                                        "API disabled or restricted - cannot validate".to_string()
                                    ))
                                }
                            } else {
                                Err(crate::core::error::KeyHunterError::ValidationFailed(
                                    "Google API returned 403".to_string()
                                ))
                            }
                        }
                        Err(_) => {
                            Err(crate::core::error::KeyHunterError::ValidationFailed(
                                "Failed to parse Google API error response".to_string()
                            ))
                        }
                    }
                } else if response.status_code == 429 {
                    // Rate limited - return error, don't mark key as invalid
                    Err(crate::core::error::KeyHunterError::RateLimit(
                        "Google API rate limit exceeded".to_string()
                    ))
                } else if response.status_code >= 500 {
                    // Server error - return error, don't mark key as invalid
                    Err(crate::core::error::KeyHunterError::ValidationFailed(
                        format!("Google API server error: HTTP {}", response.status_code)
                    ))
                } else {
                    // Other client error
                    Err(crate::core::error::KeyHunterError::ValidationFailed(
                        format!("Google API returned HTTP {}", response.status_code)
                    ))
                }
            }
            Err(e) => {
                // Network or curl error - DON'T mark key as invalid
                Err(crate::core::error::KeyHunterError::Http(
                    format!("Network error validating Google key: {}", e)
                ))
            }
        }
    }

    fn key_type(&self) -> &str {
        "google"
    }

    fn rate_limit(&self) -> Duration {
        // Google has rate limits - 2 seconds between validation requests
        Duration::from_millis(2000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_validator_creation() {
        let validator = GoogleValidator::new();
        assert_eq!(validator.key_type(), "google");
    }
}
