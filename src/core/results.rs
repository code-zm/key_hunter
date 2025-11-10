use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A key detected in source code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedKey {
    pub key: String,
    pub key_type: String,
    pub repository: String,
    pub file_path: String,
    pub file_url: String,
    pub line_number: Option<usize>,
    pub context: Option<String>, // Surrounding code
}

/// Result of validating a key against its API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub key_type: String,
    pub error: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ValidationResult {
    pub fn valid(key_type: String, metadata: HashMap<String, serde_json::Value>) -> Self {
        Self {
            valid: true,
            key_type,
            error: None,
            metadata,
        }
    }

    pub fn invalid(key_type: String, error: String) -> Self {
        Self {
            valid: false,
            key_type,
            error: Some(error),
            metadata: HashMap::new(),
        }
    }
}

/// A validated key with additional information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedKey {
    pub detected: DetectedKey,
    pub validation: ValidationResult,
    pub validated_at: DateTime<Utc>,
}

/// Search result from a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub repository: String,
    pub file_path: String,
    pub file_url: String,
    pub download_url: String,
    pub default_branch: Option<String>,
    /// Text match snippets from the search (avoids downloading full file)
    pub text_matches: Option<Vec<String>>,
}

/// Query for searching
#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub query: String,
    pub max_results: usize,
    pub file_extensions: Vec<String>,
}

/// Complete hunt results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HuntResults {
    pub timestamp: DateTime<Utc>,
    pub total_keys_found: usize,
    pub valid_keys: Vec<ValidatedKey>,
    pub invalid_keys: Vec<ValidatedKey>,
    pub by_key_type: HashMap<String, usize>,
    pub statistics: Statistics,
}

impl Default for HuntResults {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            total_keys_found: 0,
            valid_keys: Vec::new(),
            invalid_keys: Vec::new(),
            by_key_type: HashMap::new(),
            statistics: Statistics::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Statistics {
    pub files_attempted: usize,
    pub files_downloaded: usize,
    pub files_404: usize,
    pub files_other_error: usize,
    pub files_from_snippets: usize,  // Files processed using text match snippets (no download)
    pub keys_found: usize,
    pub keys_tested: usize,
    pub keys_valid: usize,
    pub keys_invalid: usize,
}

/// Report format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    Json,
    Csv,
    Html,
    Text,
}
