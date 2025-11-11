pub mod claude;
pub mod gemini;
pub mod github;
pub mod openai;
pub mod openrouter;
pub mod shodan;
pub mod xai;

pub use claude::ClaudeValidator;
pub use gemini::GeminiValidator;
pub use github::GitHubValidator;
pub use openai::OpenAIValidator;
pub use openrouter::OpenRouterValidator;
pub use shodan::ShodanValidator;
pub use xai::XAIValidator;

// Re-export for convenience
use crate::core::traits::KeyValidator;
use crate::core::config::ValidatorsConfig;
use std::collections::HashMap;

/// Get all available validators
pub fn all_validators(config: &ValidatorsConfig) -> HashMap<String, Box<dyn KeyValidator>> {
    let mut validators: HashMap<String, Box<dyn KeyValidator>> = HashMap::new();
    validators.insert("shodan".to_string(), Box::new(ShodanValidator::new(config.shodan_rate_limit_ms)));
    validators.insert("openai".to_string(), Box::new(OpenAIValidator::new(config.openai_rate_limit_ms)));
    validators.insert("openrouter".to_string(), Box::new(OpenRouterValidator::new(config.openrouter_rate_limit_ms)));
    validators.insert("claude".to_string(), Box::new(ClaudeValidator::new(config.claude_rate_limit_ms)));
    validators.insert("gemini".to_string(), Box::new(GeminiValidator::new(config.gemini_rate_limit_ms)));
    validators.insert("xai".to_string(), Box::new(XAIValidator::new(config.xai_rate_limit_ms)));
    validators.insert("github".to_string(), Box::new(GitHubValidator::new(config.github_rate_limit_ms)));
    validators
}

/// Get a validator by key type
pub fn get_validator(key_type: &str, config: &ValidatorsConfig) -> Option<Box<dyn KeyValidator>> {
    match key_type.to_lowercase().as_str() {
        "shodan" => Some(Box::new(ShodanValidator::new(config.shodan_rate_limit_ms))),
        "openai" => Some(Box::new(OpenAIValidator::new(config.openai_rate_limit_ms))),
        "openrouter" => Some(Box::new(OpenRouterValidator::new(config.openrouter_rate_limit_ms))),
        "claude" => Some(Box::new(ClaudeValidator::new(config.claude_rate_limit_ms))),
        "gemini" => Some(Box::new(GeminiValidator::new(config.gemini_rate_limit_ms))),
        "xai" => Some(Box::new(XAIValidator::new(config.xai_rate_limit_ms))),
        "github" | "github_token" => Some(Box::new(GitHubValidator::new(config.github_rate_limit_ms))),
        _ => None,
    }
}
