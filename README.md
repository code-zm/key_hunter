# Key Hunter 🔑

A modular, high-performance framework for hunting exposed API keys in code repositories. Written in Rust for speed and safety.

## ⚠️ Ethical Use Only

This tool is designed for:
- Security research and awareness
- Responsible disclosure
- Educational purposes
- Authorized security testing

**DO NOT** use this tool for:
- Unauthorized access
- Malicious purposes
- Exploiting discovered keys

By using this tool, you agree to report all findings to repository owners and service providers.

## Features

- 🚀 **Fast & Async**: Built on Tokio for concurrent operations
- 🔌 **Modular**: Easy to add new key types via trait system
- 🛡️ **Type Safe**: Rust's type system prevents common bugs
- 🌐 **Multiple Providers**: GitHub, GitLab, local file search
- ⏱️ **Rate Limited**: Built-in rate limiting respects API limits
- 📊 **Rich Output**: Beautiful terminal output with progress bars
- 🔍 **Smart Detection**: Entropy-based filtering reduces false positives
- 📁 **Auto-Organization**: Results automatically organized by key type with timestamps
- ✅ **Valid Keys Only**: Saves only verified, working keys (invalid keys not saved)

## Supported Key Types

### Cloud Providers
- ✅ **AWS** - Access Keys (AKIA*), AppSync GraphQL Keys
- ✅ **Google Cloud** - API Keys (AIza*), OAuth, Service Accounts
- ✅ **Heroku** - API Keys

### Payment Services
- ✅ **Stripe** - Live API Keys, Restricted API Keys
- ✅ **Square** - Access Tokens, OAuth Secrets
- ✅ **PayPal Braintree** - Access Tokens
- ✅ **Picatic** - API Keys

### Communication & Social
- ✅ **Slack** - Tokens (xox*), Webhooks
- ✅ **Telegram** - Bot API Keys
- ✅ **Twilio** - API Keys
- ✅ **Twitter** - Access Tokens, OAuth
- ✅ **Facebook** - Access Tokens, OAuth
- ✅ **MailChimp** - API Keys
- ✅ **Mailgun** - API Keys

### Development Platforms
- ✅ **GitHub** - Personal Access Tokens, OAuth Tokens
- ✅ **OpenAI** - API Keys (sk-*, sk-proj-*)
- ✅ **Shodan** - API Keys

### Private Keys
- ✅ **RSA Private Keys**
- ✅ **SSH Private Keys** (DSA, EC)
- ✅ **PGP Private Key Blocks**

### Generic Patterns
- ✅ **Generic API Keys** - Common API key patterns
- ✅ **Generic Secrets** - Common secret patterns
- ✅ **Passwords in URLs** - Embedded credentials

**Total: 30+ key types supported across 8 detector modules**

## Installation

### Prerequisites

- Rust 1.70+
- libcurl development libraries

```bash
# Ubuntu/Debian
sudo apt-get install libcurl4-openssl-dev

# macOS
brew install curl

# Fedora
sudo dnf install libcurl-devel
```

### Build from Source

```bash
git clone https://github.com/yourusername/key_hunter.git
cd key_hunter
cargo build --release
```

The binary will be at `target/release/key-hunter`.

## Quick Start

### 1. Search for API Keys on GitHub

```bash
# Set your GitHub token for better rate limits
export GITHUB_TOKEN=ghp_your_token_here

# Search for all key types with validation
key-hunter search --key-type all --auto-split --test-immediately

# Search for specific key type (shodan, openai, aws, stripe, slack, google, github, misc)
key-hunter search --key-type aws --auto-split --test-immediately
key-hunter search --key-type stripe --test-immediately
key-hunter search --key-type slack --test-immediately

# Search without immediate validation (faster)
key-hunter search --key-type openai --auto-split

# Custom query for specific patterns
key-hunter search --query "SHODAN_API_KEY extension:env" --key-type shodan
key-hunter search --query "sk_live_ extension:py" --key-type stripe
```

### 2. Validate Keys from File

```bash
# Validate previously found keys
key-hunter validate --input results.json --output validated.json
```

### 3. Test a Single Key

```bash
# Test a specific key
key-hunter test oykKBEq2KRySU33OxizNkOir5PgHpMLv --key-type shodan
```

### 4. Generate Reports

```bash
# Generate text report
key-hunter report --input results.json --format text

# Save to file
key-hunter report --input results.json --format json --output report.json
```

### 5. List Available Detectors

```bash
key-hunter list detectors
```

## Output Organization

Key Hunter automatically organizes your results for easy management:

### Default Output Structure

When you run a search, results are automatically saved to:
```
results/<key-type>/valid_keys_<timestamp>.json
```

**Examples:**
- `results/shodan/valid_keys_20251108_143022.json`
- `results/openai/valid_keys_20251108_150430.json`
- `results/aws/valid_keys_20251108_162145.json`

### Key Features

- **Automatic directories**: `results/<key-type>/` directories are created automatically
- **Timestamped files**: Each scan gets a unique filename with timestamp
- **Valid keys only**: Only verified, working keys are saved (no invalid/rotated keys)
- **No overwrites**: Previous scans are never overwritten
- **Custom output**: Use `-o` flag to specify a custom output path

### Output Format

Each results file contains:
```json
{
  "timestamp": "2025-11-08T19:44:57Z",
  "key_type": "shodan",
  "total_valid_keys": 168,
  "total_keys_scanned": 247,
  "valid_keys": [
    {
      "detected": {
        "key": "EBUfD8F...",
        "key_type": "shodan",
        "repository": "username/repo",
        "file_path": "config.env",
        "file_url": "https://github.com/...",
        "line_number": 4
      },
      "validation": {
        "valid": true,
        "key_type": "shodan",
        "metadata": {
          "plan": "dev",
          "scan_credits": 100,
          "query_credits": 100
        }
      }
    }
  ]
}
```

## Architecture

Key Hunter is built around three core traits:

### KeyDetector
Detects potential keys using regex patterns and entropy filtering.

```rust
pub trait KeyDetector: Send + Sync {
    fn name(&self) -> &str;
    fn detect(&self, content: &str, file_path: &str) -> Vec<DetectedKey>;
    fn patterns(&self) -> &[Regex];
    fn search_queries(&self) -> Vec<String>;
}
```

### KeyValidator
Validates detected keys against actual APIs.

```rust
#[async_trait]
pub trait KeyValidator: Send + Sync {
    async fn validate(&self, key: &str) -> Result<ValidationResult>;
    fn key_type(&self) -> &str;
    fn rate_limit(&self) -> Duration;
}
```

### SearchProvider
Searches code repositories for exposed keys.

```rust
#[async_trait]
pub trait SearchProvider: Send + Sync {
    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>>;
    fn name(&self) -> &str;
    async fn get_file_content(&self, result: &SearchResult) -> Result<String>;
}
```

## Adding New Key Types

Adding a new key type is easy! Just implement two traits:

### 1. Create a Detector

```rust
// src/detectors/myservice.rs
use crate::core::{KeyDetector, DetectedKey};
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref MY_PATTERN: Regex = Regex::new(r"myservice_[A-Za-z0-9]{32}").unwrap();
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
        // Implementation here
        vec![]
    }

    fn patterns(&self) -> &[Regex] {
        &self.patterns
    }

    fn search_queries(&self) -> Vec<String> {
        vec!["MYSERVICE_API_KEY".to_string()]
    }

    fn file_extensions(&self) -> &[&str] {
        &[".env", ".py", ".js"]
    }
}
```

### 2. Create a Validator

```rust
// src/validators/myservice.rs
use crate::core::{KeyValidator, ValidationResult};
use async_trait::async_trait;

pub struct MyServiceValidator;

#[async_trait]
impl KeyValidator for MyServiceValidator {
    async fn validate(&self, key: &str) -> Result<ValidationResult> {
        // Call API to validate key
        Ok(ValidationResult::valid("myservice".to_string(), metadata))
    }

    fn key_type(&self) -> &str {
        "myservice"
    }
}
```

### 3. Register Your Types

Add them to `src/detectors/mod.rs` and `src/validators/mod.rs`.

## Configuration

### Environment Variables

Create a `.env` file for sensitive credentials:

```bash
# .env
GITHUB_TOKEN=ghp_your_token_here
```

### Config File

Key Hunter automatically loads settings from `config/default.toml`. You can customize:

- **Rate limiting**: Adjust `rate_limit_delay_ms` to control request speed
- **API endpoints**: Change `base_url` for custom GitHub Enterprise instances
- **Output settings**: Default format and directory
- **Detector settings**: Enable/disable specific key detectors

```toml
[github]
base_url = "https://api.github.com"
# Rate limit delay in milliseconds between requests
# With token: 3000ms (20 req/min) - avoids rate limit errors
# Without token: 6000ms (10 req/min)
rate_limit_delay_ms = 3000

[output]
format = "json"
directory = "./output"
save_invalid = true

[detectors.shodan]
enabled = true
```

**Important:** GitHub's Code Search API has strict rate limits. The default `3000ms` delay ensures you stay under the limit and avoid constant 60-second waits. Don't lower this value unless you want to hit rate limits frequently.

Config file locations (in order of priority):
1. `config/default.toml` (recommended)
2. `default.toml`
3. `.key_hunter.toml`

## CLI Reference

### Search Command
```bash
key-hunter search [OPTIONS]

Options:
  -p, --provider <PROVIDER>          Search provider (github, gitlab, local) [default: github]
  -k, --key-type <KEY_TYPE>          Key type to search for [default: all]
  -q, --query <QUERY>                Custom search query
  -m, --max-results <MAX_RESULTS>    Max results per query [default: 1000]
  -n, --num-queries <NUM_QUERIES>    Number of queries to run [default: 3]
  -o, --output <OUTPUT>              Output file [default: results/<key-type>/valid_keys_<timestamp>.json]
  -t, --test-immediately             Test keys as they're found
      --auto-split                   Split queries by file type to bypass 1000 result limit
      --github-token <TOKEN>         GitHub token

Note: By default, only valid keys are saved. Invalid/rotated keys are not written to disk.
```

### Validate Command
```bash
key-hunter validate --input <FILE> --output <FILE> [--key-type <TYPE>]
```

### Test Command
```bash
key-hunter test <KEY> --key-type <TYPE>
```

### Report Command
```bash
key-hunter report --input <FILE> [--format <FORMAT>] [--output <FILE>]
```

### List Command
```bash
key-hunter list [detectors|validators|all]
```

## Development

### Running Tests

```bash
cargo test
```

### Running with Verbose Logging

```bash
key-hunter --verbose search --key-type shodan
```

## Project Structure

```
key_hunter/
├── src/
│   ├── core/           # Core traits and types
│   ├── detectors/      # Key detection modules
│   ├── validators/     # Key validation modules
│   ├── providers/      # Search providers
│   ├── utils/          # HTTP client, rate limiting
│   └── cli/            # CLI interface
├── config/             # Configuration files
├── tests/              # Integration tests
└── examples/           # Example usage
```

## Performance

Key Hunter is designed for performance:

- **Async I/O**: All network operations are non-blocking
- **Parallel Processing**: Multiple files processed concurrently
- **Smart Caching**: HTTP responses cached when appropriate
- **Optimized Regex**: Compiled patterns reused

## Responsible Disclosure

If you find valid keys:

1. **Report to repository owners** via GitHub issues or security email
2. **Report to service providers** (e.g., support@shodan.io)
3. **Document your findings** for security awareness
4. **DO NOT use keys** for unauthorized access

## License

MIT License - see LICENSE file for details

## Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Submit a pull request

## Credits

Inspired by the original Python implementation for Shodan key research.

Built with:
- [Tokio](https://tokio.rs/) - Async runtime
- [curl-rust](https://github.com/alexcrichton/curl-rust) - HTTP client
- [Clap](https://github.com/clap-rs/clap) - CLI framework
- [Serde](https://serde.rs/) - Serialization

## Disclaimer

This tool is provided for educational and research purposes only. The authors are not responsible for any misuse or damage caused by this tool. Always obtain proper authorization before testing systems you do not own.
