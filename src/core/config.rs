use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub github: Option<GitHubConfig>,
    pub output: OutputConfig,
    pub detectors: HashMap<String, DetectorConfig>,
    pub validators: Option<ValidatorsConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            github: None,
            output: OutputConfig::default(),
            detectors: HashMap::new(),
            validators: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    pub token: Option<String>,
    pub base_url: String,
    pub rate_limit_delay_ms: u64,
}

impl Default for GitHubConfig {
    fn default() -> Self {
        Self {
            token: None,
            base_url: "https://api.github.com".to_string(),
            rate_limit_delay_ms: 2000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub format: String,
    pub directory: String,
    pub save_invalid: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: "json".to_string(),
            directory: "./output".to_string(),
            save_invalid: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectorConfig {
    pub enabled: bool,
    pub patterns: Option<Vec<String>>,
    pub search_queries: Option<Vec<String>>,
    pub file_extensions: Option<Vec<String>>,
}

impl DetectorConfig {
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            patterns: None,
            search_queries: None,
            file_extensions: None,
        }
    }

    pub fn disabled() -> Self {
        Self {
            enabled: false,
            patterns: None,
            search_queries: None,
            file_extensions: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorsConfig {
    pub openai_rate_limit_ms: u64,
    pub claude_rate_limit_ms: u64,
    pub gemini_rate_limit_ms: u64,
    pub shodan_rate_limit_ms: u64,
    pub xai_rate_limit_ms: u64,
    pub openrouter_rate_limit_ms: u64,
    pub github_rate_limit_ms: u64,
}

impl Default for ValidatorsConfig {
    fn default() -> Self {
        Self {
            openai_rate_limit_ms: 1000,      // 60 RPM - conservative for free tier
            claude_rate_limit_ms: 2000,      // 30 RPM - Tier 1 limit
            gemini_rate_limit_ms: 2000,      // 30 RPM - conservative for paid tier
            shodan_rate_limit_ms: 1000,      // 60 RPM - enforced 1 req/sec
            xai_rate_limit_ms: 1000,         // 60 RPM - conservative
            openrouter_rate_limit_ms: 3000,  // 20 RPM - free tier limit
            github_rate_limit_ms: 2000,      // 30 RPM - secondary rate limit safe
        }
    }
}
