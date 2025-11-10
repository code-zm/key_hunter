//! # Key Hunter
//!
//! A modular framework for hunting API keys in code repositories.
//!
//! ## Features
//!
//! - **Modular**: Easy to add new key types via traits
//! - **Async**: Parallel validation and searching with Tokio
//! - **Rate-limited**: Built-in rate limiting per provider
//! - **Configurable**: TOML-based configuration
//! - **Multiple Providers**: GitHub, GitLab, local file search
//!
//! ## Architecture
//!
//! The framework is built around three main traits:
//!
//! - `KeyDetector`: Detects potential keys using regex patterns
//! - `KeyValidator`: Validates keys against actual APIs
//! - `SearchProvider`: Searches code repositories
//!
//! ## Example
//!
//! ```rust,no_run
//! use key_hunter::detectors::ShodanDetector;
//! use key_hunter::core::KeyDetector;
//!
//! let detector = ShodanDetector::new();
//! let content = "SHODAN_API_KEY=oykKBEq2KRySU33OxizNkOir5PgHpMLv";
//! let keys = detector.detect(content, "test.env");
//!
//! println!("Found {} keys", keys.len());
//! ```

pub mod cli;
pub mod core;
pub mod detectors;
pub mod providers;
pub mod utils;
pub mod validators;

// Re-export commonly used types
pub use core::{
    Config, DetectedKey, HuntResults, KeyDetector, KeyHunterError, KeyValidator, ReportFormat,
    Result, SearchProvider, SearchQuery, SearchResult, ValidatedKey, ValidationResult,
};

pub use detectors::{all_detectors, get_detector};
pub use providers::GitHubProvider;
pub use validators::{all_validators, get_validator};
