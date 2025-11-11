# Extending Key Hunter

Guide for developers who want to add support for new key types.

---

## Architecture Overview

Key Hunter is built around a trait-based architecture with three core traits:

### 1. KeyDetector
Detects potential keys using regex patterns and entropy filtering.

```rust
pub trait KeyDetector: Send + Sync {
    fn name(&self) -> &str;
    fn detect(&self, content: &str, file_path: &str) -> Vec<DetectedKey>;
    fn patterns(&self) -> &[Regex];
    fn search_queries(&self) -> Vec<String>;
    fn file_extensions(&self) -> &[&str];
}
```

### 2. KeyValidator
Validates detected keys against actual APIs.

```rust
#[async_trait]
pub trait KeyValidator: Send + Sync {
    async fn validate(&self, key: &str) -> Result<ValidationResult>;
    fn key_type(&self) -> &str;
    fn rate_limit(&self) -> Duration;
}
```

### 3. SearchProvider
Searches code repositories for exposed keys.

```rust
#[async_trait]
pub trait SearchProvider: Send + Sync {
    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>>;
    fn name(&self) -> &str;
    async fn get_file_content(&self, result: &SearchResult) -> Result<String>;
}
```

---

## Adding a New Key Type

### Step 1: Create the Detector

Create a new file `src/detectors/myservice.rs`:

```rust
use crate::core::{KeyDetector, DetectedKey};
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    // Define your regex pattern(s)
    static ref MY_PATTERN: Regex =
        Regex::new(r"myservice_[A-Za-z0-9]{32}").unwrap();
}

pub struct MyServiceDetector {
    patterns: Vec<Regex>,
}

impl MyServiceDetector {
    pub fn new() -> Self {
        Self {
            patterns: vec![MY_PATTERN.clone()],
        }
    }
}

impl KeyDetector for MyServiceDetector {
    fn name(&self) -> &str {
        "myservice"
    }

    fn detect(&self, content: &str, file_path: &str) -> Vec<DetectedKey> {
        let mut detected = Vec::new();

        for pattern in &self.patterns {
            for capture in pattern.captures_iter(content) {
                if let Some(matched) = capture.get(0) {
                    let key = matched.as_str().to_string();

                    // Optional: Add entropy filtering to reduce false positives
                    // if calculate_entropy(&key) < 3.5 {
                    //     continue;
                    // }

                    detected.push(DetectedKey {
                        key,
                        key_type: self.name().to_string(),
                        file_path: file_path.to_string(),
                        line_number: None, // Can calculate if needed
                        repository: String::new(), // Filled in by search provider
                        file_url: String::new(), // Filled in by search provider
                    });
                }
            }
        }

        detected
    }

    fn patterns(&self) -> &[Regex] {
        &self.patterns
    }

    fn search_queries(&self) -> Vec<String> {
        // GitHub search queries to find this key type
        vec![
            "MYSERVICE_API_KEY".to_string(),
            "MYSERVICE_SECRET".to_string(),
        ]
    }

    fn file_extensions(&self) -> &[&str] {
        // Common file types where keys might appear
        &[".env", ".config", ".py", ".js", ".ts", ".go", ".rs"]
    }
}
```

### Step 2: Create the Validator

Create `src/validators/myservice.rs`:

```rust
use crate::core::{KeyValidator, ValidationResult};
use crate::utils::HttpClient;
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Duration;

pub struct MyServiceValidator;

impl MyServiceValidator {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl KeyValidator for MyServiceValidator {
    async fn validate(&self, key: &str) -> crate::core::Result<ValidationResult> {
        // Make API call to verify the key
        let response = tokio::task::spawn_blocking({
            let client = HttpClient::new();
            let key = key.to_string();
            move || {
                let url = format!("https://api.myservice.com/validate");
                let headers = vec![
                    ("Authorization", format!("Bearer {}", key)),
                ];
                let header_refs: Vec<(&str, &str)> = headers
                    .iter()
                    .map(|(k, v)| (*k, v.as_str()))
                    .collect();

                client.get(&url, &header_refs)
            }
        })
        .await
        .map_err(|e| crate::core::KeyHunterError::Unknown(
            format!("Task join error: {}", e)
        ))??;

        if response.is_success() {
            // Parse response for metadata
            let mut metadata = HashMap::new();

            // Example: extract plan type, credits, etc.
            if let Ok(body) = response.text() {
                // Parse JSON or extract info
                metadata.insert("status".to_string(), "active".to_string());
            }

            Ok(ValidationResult::valid(
                self.key_type().to_string(),
                metadata,
            ))
        } else if response.status_code == 401 || response.status_code == 403 {
            // Invalid key
            Ok(ValidationResult::invalid(self.key_type().to_string()))
        } else {
            // Other error
            Err(crate::core::KeyHunterError::Validation(format!(
                "API returned {}: {}",
                response.status_code,
                response.text().unwrap_or_default()
            )))
        }
    }

    fn key_type(&self) -> &str {
        "myservice"
    }

    fn rate_limit(&self) -> Duration {
        // Delay between validation requests
        Duration::from_millis(1000)
    }
}
```

### Step 3: Register the Detector

In `src/detectors/mod.rs`, add:

```rust
mod myservice;

pub use myservice::MyServiceDetector;

pub fn all_detectors() -> Vec<Box<dyn KeyDetector>> {
    vec![
        // ... existing detectors
        Box::new(MyServiceDetector::new()),
    ]
}

pub fn get_detector(key_type: &str) -> Option<Box<dyn KeyDetector>> {
    match key_type {
        // ... existing cases
        "myservice" => Some(Box::new(MyServiceDetector::new())),
        _ => None,
    }
}
```

### Step 4: Register the Validator

In `src/validators/mod.rs`, add:

```rust
mod myservice;

pub use myservice::MyServiceValidator;

pub fn all_validators() -> HashMap<String, Box<dyn KeyValidator>> {
    let mut validators = HashMap::new();
    // ... existing validators
    validators.insert(
        "myservice".to_string(),
        Box::new(MyServiceValidator::new()) as Box<dyn KeyValidator>
    );
    validators
}
```

### Step 5: Add Issue Template Configuration (Optional)

In `src/reporters/mod.rs`, add service-specific configuration:

```rust
impl ServiceConfig {
    fn get(key_type: &str) -> Self {
        match key_type {
            // ... existing services
            "myservice" => Self {
                service_name: "MyService".to_string(),
                revoke_url: "https://myservice.com/settings/api-keys".to_string(),
                additional_actions: "".to_string(),
                best_practices: "".to_string(),
                resources: "- [MyService API Keys](https://myservice.com/docs/api)\n".to_string(),
            },
            _ => // ... default
        }
    }
}
```

---

## Testing Your Implementation

### 1. Test Detection

```bash
# Create a test file with your key pattern
echo "myservice_abc123xyz456def789ghi012jkl345" > test.env

# Test detection (you'll need to implement a test mode or use the test command)
cargo test -- myservice_detector
```

### 2. Test Validation

```bash
# Build and test with a real key
cargo build --release
./target/release/key-hunter test "myservice_realkey123..." --key-type myservice
```

### 3. Test Search

```bash
# Search GitHub for your key type
./target/release/key-hunter search --key-type myservice --validate
```

---

## Best Practices

### 1. Regex Patterns

- **Be specific** - Avoid overly broad patterns that match non-keys
- **Use anchors** when possible (word boundaries, etc.)
- **Test extensively** - Use regex101.com or similar
- **Consider entropy** - Filter out low-entropy strings

Example:
```rust
// Good - specific pattern
r"myservice_[A-Za-z0-9]{32}"

// Bad - too broad
r"[A-Za-z0-9]{32}"
```

### 2. Validation

- **Handle rate limits** - Implement appropriate delays
- **Parse errors carefully** - Don't confuse network errors with invalid keys
- **Extract metadata** - Return useful info (plan type, credits, etc.)
- **Test edge cases** - Expired keys, revoked keys, malformed keys

### 3. Search Queries

- **Use service-specific terms** - Environment variable names, config keys
- **Include common variations** - `API_KEY`, `SECRET_KEY`, `TOKEN`, etc.
- **Target relevant file types** - Focus on config files, code files

### 4. Error Handling

Always use proper error types:

```rust
// Good
Err(KeyHunterError::Validation("Invalid API response".to_string()))

// Bad
panic!("API call failed");
```

### 5. Async/Blocking

HTTP calls using curl-rust must be wrapped in `spawn_blocking`:

```rust
let response = tokio::task::spawn_blocking({
    let client = HttpClient::new();
    let url = url.clone();
    move || client.get(&url, &[])
})
.await
.map_err(|e| KeyHunterError::Unknown(format!("Task join error: {}", e)))??;
```

---

## Common Patterns

### Pattern: Multi-Pattern Detection

Some services have multiple key formats:

```rust
lazy_static! {
    static ref PATTERN_V1: Regex = Regex::new(r"myservice_v1_[A-Z0-9]{32}").unwrap();
    static ref PATTERN_V2: Regex = Regex::new(r"myservice_v2_[a-z0-9]{64}").unwrap();
}

impl MyServiceDetector {
    pub fn new() -> Self {
        Self {
            patterns: vec![PATTERN_V1.clone(), PATTERN_V2.clone()],
        }
    }
}
```

### Pattern: API Key + Secret

Some services use key+secret pairs:

```rust
// Detect both and store in metadata
let mut metadata = HashMap::new();
metadata.insert("api_key".to_string(), key.to_string());
metadata.insert("api_secret".to_string(), secret.to_string());
```

### Pattern: Composite Validation

Some keys need multiple API calls:

```rust
async fn validate(&self, key: &str) -> Result<ValidationResult> {
    // First, check if key exists
    let exists = self.check_exists(key).await?;
    if !exists {
        return Ok(ValidationResult::invalid(self.key_type().to_string()));
    }

    // Then, check permissions
    let perms = self.check_permissions(key).await?;

    let mut metadata = HashMap::new();
    metadata.insert("permissions".to_string(), perms);

    Ok(ValidationResult::valid(self.key_type().to_string(), metadata))
}
```

---

## Example: Complete Implementation

See the existing implementations for reference:

- **Simple API key**: `src/detectors/shodan.rs` + `src/validators/shodan.rs`
- **Multiple patterns**: `src/detectors/openai.rs` (handles `sk-*` and `sk-proj-*`)
- **Complex validation**: `src/validators/claude.rs` (extracts quota info)

---

## Debugging Tips

### Enable verbose logging

```bash
./target/release/key-hunter --verbose search --key-type myservice
```

### Test regex patterns

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_matching() {
        let detector = MyServiceDetector::new();
        let content = "MYSERVICE_API_KEY=myservice_abc123xyz456def789ghi012jkl345";
        let detected = detector.detect(content, "test.env");

        assert_eq!(detected.len(), 1);
        assert_eq!(detected[0].key, "myservice_abc123xyz456def789ghi012jkl345");
    }

    #[test]
    fn test_no_false_positives() {
        let detector = MyServiceDetector::new();
        let content = "not_a_key_12345";
        let detected = detector.detect(content, "test.env");

        assert_eq!(detected.len(), 0);
    }
}
```

### Test validation manually

```rust
#[tokio::test]
async fn test_validation() {
    let validator = MyServiceValidator::new();
    let result = validator.validate("test_key_here").await;

    match result {
        Ok(validation) => println!("Valid: {}", validation.valid),
        Err(e) => println!("Error: {}", e),
    }
}
```

---

## Contribution Guidelines

When adding a new key type:

1. **Create a pull request** with your implementation
2. **Include tests** for both detection and validation
3. **Document the pattern** - Explain the key format
4. **Test with real keys** - Verify validation works correctly
5. **Update documentation** - Add to supported services list

---

## Questions?

- Open an issue on GitHub
- Check existing implementations for examples
- Review the core traits in `src/core/`
