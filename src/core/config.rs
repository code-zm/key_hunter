use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub github: Option<GitHubConfig>,
    pub output: OutputConfig,
    pub detectors: HashMap<String, DetectorConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            github: None,
            output: OutputConfig::default(),
            detectors: HashMap::new(),
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
