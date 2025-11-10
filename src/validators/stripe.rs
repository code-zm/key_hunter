use crate::core::error::Result;
use crate::core::results::ValidationResult;
use crate::core::traits::KeyValidator;
use crate::utils::HttpClient;
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct StripeBalance {
    available: Option<Vec<BalanceAmount>>,
    livemode: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct BalanceAmount {
    currency: String,
}

pub struct StripeValidator;

impl StripeValidator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StripeValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl KeyValidator for StripeValidator {
    async fn validate(&self, key: &str) -> Result<ValidationResult> {
        let url = "https://api.stripe.com/v1/balance";

        // Perform request in blocking context (curl is sync)
        let result = tokio::task::spawn_blocking({
            let client = HttpClient::new();
            let auth_header = format!("Bearer {}", key);
            move || {
                client.get(
                    url,
                    &[
                        ("Authorization", &auth_header),
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
                    match response.json::<StripeBalance>() {
                        Ok(balance) => {
                            let mut metadata = HashMap::new();

                            if let Some(livemode) = balance.livemode {
                                metadata.insert(
                                    "livemode".to_string(),
                                    serde_json::Value::Bool(livemode),
                                );
                            }

                            if let Some(available) = balance.available {
                                if let Some(first) = available.first() {
                                    metadata.insert(
                                        "currency".to_string(),
                                        serde_json::Value::String(first.currency.clone()),
                                    );
                                }
                            }

                            Ok(ValidationResult::valid("stripe".to_string(), metadata))
                        }
                        Err(_) => {
                            // Invalid response format but 200 status - still valid
                            let mut metadata = HashMap::new();
                            metadata.insert(
                                "note".to_string(),
                                serde_json::Value::String("Valid key (200 OK)".to_string()),
                            );
                            Ok(ValidationResult::valid("stripe".to_string(), metadata))
                        }
                    }
                } else if response.status_code == 401 {
                    // ONLY 401 means invalid key
                    Ok(ValidationResult::invalid(
                        "stripe".to_string(),
                        "Unauthorized - key is invalid or revoked".to_string(),
                    ))
                } else if response.status_code == 429 {
                    // Rate limited - return error, don't mark key as invalid
                    Err(crate::core::error::KeyHunterError::RateLimit(
                        "Stripe API rate limit exceeded".to_string()
                    ))
                } else if response.status_code >= 500 {
                    // Server error - return error, don't mark key as invalid
                    Err(crate::core::error::KeyHunterError::ValidationFailed(
                        format!("Stripe API server error: HTTP {}", response.status_code)
                    ))
                } else {
                    // Other client error
                    Err(crate::core::error::KeyHunterError::ValidationFailed(
                        format!("Stripe API returned HTTP {}", response.status_code)
                    ))
                }
            }
            Err(e) => {
                // Network or curl error - DON'T mark key as invalid
                Err(crate::core::error::KeyHunterError::Http(
                    format!("Network error validating Stripe key: {}", e)
                ))
            }
        }
    }

    fn key_type(&self) -> &str {
        "stripe"
    }

    fn rate_limit(&self) -> Duration {
        // Stripe has rate limits - 1.5 seconds between validation requests
        Duration::from_millis(1500)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stripe_validator_creation() {
        let validator = StripeValidator::new();
        assert_eq!(validator.key_type(), "stripe");
    }
}
