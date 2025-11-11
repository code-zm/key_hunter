<div align="center">

<img src="keyhunter.png" alt="Key Hunter Logo" width="200"/>

# Key Hunter

**Fast, modular framework for hunting exposed API keys in code repositories**

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange?logo=rust)](https://www.rust-lang.org)
[![License: GPL 3.0](https://img.shields.io/badge/License-GPL%203.0-blue.svg)](./LICENSE)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-lightgrey)]()
[![Status](https://img.shields.io/badge/status-active-brightgreen)]()
[![GitHub Issues](https://img.shields.io/github/issues/code-zm/key_hunter)](https://github.com/code-zm/key_hunter/issues)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/code-zm/key_hunter/pulls)

[Documentation](./docs/)

</div>

---

## Ethical Use Only

This tool is designed **exclusively** for:
- Security research and vulnerability disclosure
- Educational purposes
- Authorized security testing with explicit permission

**Strictly prohibited:**
- Unauthorized access to systems or data
- Exploiting discovered credentials
- Any malicious or illegal activities

**By using this tool, you agree to responsibly disclose all findings to repository owners.**

---

## Features

- **Smart Detection** - Regex patterns with entropy filtering to reduce false positives
- **Live Validation** - Verifies keys against actual APIs in real-time
- **Auto-Reporting** - Automatically creates GitHub issues for exposed keys
- **Organized Output** - Results saved by key type with timestamps
- **Configurable Rate Limits** - Adjust validation speed based on your API tier

---

## Supported Services

- **Shodan**
- **OpenAI**
- **OpenRouter**
- **Claude (Anthropic)**
- **Gemini**
- **GitHub**
- **xAI**

---

## Installation

### Prerequisites

- Rust 1.70 or later
- libcurl development libraries

```bash
# Ubuntu/Debian
sudo apt install libcurl4-openssl-dev

# macOS
brew install curl

# Fedora/RHEL
sudo dnf install libcurl-devel
```

### Build from Source

```bash
git clone https://github.com/code-zm/key_hunter.git
cd key_hunter
cargo build --release
```

Binary will be located at `target/release/key-hunter`

---

## Quick Start

### 1. Configure Tokens

Create a `.env` file:

```bash
# GitHub token for searching
GITHUB_TOKEN1=ghp_your_token_here

# GitHub token for creating issues (optional)
ISSUES_GITHUB_TOKEN=ghp_your_token_here
```

### 2. Search and Validate

```bash
# Search for keys and validate them
./target/release/key-hunter search --key-type shodan --validate

# Search all supported key types
./target/release/key-hunter search --key-type all --validate
```

### 3. Create Issues

```bash
# Preview issues before creating them
./target/release/key-hunter report --dry-run

# Create GitHub issues for exposed keys
./target/release/key-hunter report
```

---

## Documentation

See [docs/usage.md](./docs/usage.md) for complete command reference and usage examples.

**Available commands:**
- `search` - Search GitHub for exposed API keys
- `validate` - Validate keys from a file
- `test` - Test a single key
- `report` - Create GitHub issues for exposed keys
- `list` - List available detectors and validators

---

## Configuration

**Rate Limits:** Each validator has configurable rate limits based on API restrictions:
- OpenAI: 1000ms (60 RPM)
- Claude: 2000ms (30 RPM)
- Gemini: 2000ms (30 RPM)
- Shodan: 1000ms (60 RPM)
- xAI: 1000ms (60 RPM)
- OpenRouter: 3000ms (20 RPM)
- GitHub: 2000ms (30 RPM)

Rate limits can be customized in `config/default.toml` under the `[validators]` section.

**GitHub Search Rate Limits:**
- Authenticated: 30 requests/minute per token
- Unauthenticated: 10 requests/minute per IP

See [docs/usage.md](./docs/usage.md) for complete configuration details.

---

## Output

Results are automatically organized:

```
results/
├── shodan/valid_keys_20251110_143022.json
├── openai/valid_keys_20251110_151500.json
└── claude/valid_keys_20251110_160000.json
```

**Only valid keys are saved.** Invalid or expired keys are not written to disk.

---

## Responsible Disclosure

If you discover exposed keys:

1. **Immediately report** to repository owner
2. **Do NOT** use the keys for any purpose
3. **Do NOT** publicly disclose until owner has time to respond

Key Hunter's `report` command automates step 1 by creating an issue in the affected repository.

---

## License

GPL 3.0 License - See [LICENSE](LICENSE) file for details

---

## Credits

Built with:
- [Tokio](https://tokio.rs/) - Async runtime
- [curl-rust](https://github.com/alexcrichton/curl-rust) - HTTP client
- [Clap](https://github.com/clap-rs/clap) - CLI framework
- [Serde](https://serde.rs/) - Serialization
- [indicatif](https://github.com/console-rs/indicatif) - Progress bars

---

## Disclaimer

**This tool is provided for educational and authorized security research purposes only.**

The authors assume no liability for misuse or damage caused by this tool. Users are solely responsible for ensuring they have proper authorization before testing any systems. Unauthorized access to computer systems is illegal under the Computer Fraud and Abuse Act and similar laws worldwide.

Always:
- Obtain explicit written permission before testing systems you don't own
- Comply with all applicable laws and regulations
- Follow responsible disclosure practices
- Respect rate limits and terms of service

---

<div align="center">

**Made with ❤️ for the security community**

[Report Bug](https://github.com/code-zm/key_hunter/issues) •
[Request Feature](https://github.com/code-zm/key_hunter/issues)

</div>
