use crate::core::error::Result;
use crate::core::results::ValidationResult;
use crate::core::traits::KeyValidator;
use crate::utils::HttpClient;
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct ShodanApiInfo {
    plan: Option<String>,
    query_credits: Option<i64>,
    scan_credits: Option<i64>,
    https: Option<bool>,
}

pub struct ShodanValidator {
    rate_limit_ms: u64,
}

impl ShodanValidator {
    pub fn new(rate_limit_ms: u64) -> Self {
        Self { rate_limit_ms }
    }
}

impl Default for ShodanValidator {
    fn default() -> Self {
        Self::new(1000)
    }
}

#[async_trait]
impl KeyValidator for ShodanValidator {
    async fn validate(&self, key: &str) -> Result<ValidationResult> {
        let url = format!("https://api.shodan.io/api-info?key={}", key);

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
                    // Try to parse the response
                    match response.json::<ShodanApiInfo>() {
                        Ok(info) => {
                            let mut metadata = HashMap::new();

                            if let Some(plan) = &info.plan {
                                metadata.insert(
                                    "plan".to_string(),
                                    serde_json::Value::String(plan.clone()),
                                );
                            }

                            if let Some(credits) = info.query_credits {
                                metadata.insert(
                                    "query_credits".to_string(),
                                    serde_json::Value::Number(credits.into()),
                                );
                            }

                            if let Some(credits) = info.scan_credits {
                                metadata.insert(
                                    "scan_credits".to_string(),
                                    serde_json::Value::Number(credits.into()),
                                );
                            }

                            if let Some(https) = info.https {
                                metadata.insert(
                                    "https".to_string(),
                                    serde_json::Value::Bool(https),
                                );
                            }

                            Ok(ValidationResult::valid("shodan".to_string(), metadata))
                        }
                        Err(_) => {
                            // Invalid response format - could be rate limit HTML page
                            // Don't mark as invalid, return error instead
                            Err(crate::core::error::KeyHunterError::ValidationFailed(
                                "Failed to parse Shodan API response (possible rate limit)".to_string()
                            ))
                        }
                    }
                } else if response.status_code == 401 {
                    // ONLY 401 means invalid key
                    Ok(ValidationResult::invalid(
                        "shodan".to_string(),
                        "Unauthorized - key is invalid or revoked".to_string(),
                    ))
                } else if response.status_code == 429 {
                    // Rate limited - return error, don't mark key as invalid
                    Err(crate::core::error::KeyHunterError::RateLimit(
                        "Shodan API rate limit exceeded".to_string()
                    ))
                } else if response.status_code >= 500 {
                    // Server error - return error, don't mark key as invalid
                    Err(crate::core::error::KeyHunterError::ValidationFailed(
                        format!("Shodan API server error: HTTP {}", response.status_code)
                    ))
                } else {
                    // Other client error - could be invalid but might be temporary
                    Err(crate::core::error::KeyHunterError::ValidationFailed(
                        format!("Shodan API returned HTTP {}", response.status_code)
                    ))
                }
            }
            Err(e) => {
                // Network or curl error - DON'T mark key as invalid
                // Return error so calling code can retry or skip
                Err(crate::core::error::KeyHunterError::Http(
                    format!("Network error validating Shodan key: {}", e)
                ))
            }
        }
    }

    fn key_type(&self) -> &str {
        "shodan"
    }

    fn rate_limit(&self) -> Duration {
        Duration::from_millis(self.rate_limit_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shodan_validator_creation() {
        let validator = ShodanValidator::default();
        assert_eq!(validator.key_type(), "shodan");
    }

    #[tokio::test]
    async fn test_validate_invalid_key() {
        let validator = ShodanValidator::default();
        let result = validator.validate("invalidshodankey1234567890ab").await;

        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(!validation.valid);
    }
}
