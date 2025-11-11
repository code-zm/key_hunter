# Key Hunter Usage Guide

Complete documentation for using Key Hunter.

---

## Commands

### Search Command

Search GitHub for exposed API keys.

```bash
key-hunter search [OPTIONS]

Options:
  -p, --provider <PROVIDER>    Search provider [default: github]
  -k, --key-type <KEY_TYPE>    Key type to search [default: all]
  -q, --query <QUERY>          Custom search query (overrides defaults)
  -o, --output <OUTPUT>        Output file path
      --validate               Validate keys immediately as found
  -v, --verbose                Show detailed logs
```

**Examples:**
```bash
# Search for Claude keys with validation
key-hunter search -k claude --validate

# Custom query
key-hunter search -k shodan -q "SHODAN_API_KEY extension:env"

# Save to custom location
key-hunter search -k openai -o my_results.json

# Search all key types with validation
key-hunter search --key-type all --validate
```

**How it works:**
- Searches GitHub Code Search API across 77 different file types
- Automatically rotates through available GitHub tokens when rate limited
- Progress bar shows current file type and total results found
- With `--validate`: spinner shows validation progress and valid key count
- Results saved to `results/<key-type>/valid_keys_<timestamp>.json`

---

### Validate Command

Validate keys from a previously saved results file.

```bash
key-hunter validate --input <FILE> --output <FILE> [OPTIONS]

Options:
  -i, --input <FILE>           Input file with keys to validate
  -o, --output <FILE>          Output file [default: validated_results.json]
  -k, --key-type <KEY_TYPE>    Filter by key type [default: all]
  -v, --verbose                Show detailed logs
```

**Example:**
```bash
# Validate all keys from a file
key-hunter validate -i results/shodan/keys_20251110.json -o validated.json

# Validate only specific key type
key-hunter validate -i all_keys.json -k shodan -o shodan_validated.json
```

---

### Test Command

Test a single key to verify if it's valid.

```bash
key-hunter test <KEY> --key-type <TYPE>

Options:
  -k, --key-type <TYPE>    Key type (required)
```

**Examples:**
```bash
# Test a Shodan key
key-hunter test "ABC123XYZ..." --key-type shodan

# Test an OpenAI key
key-hunter test "sk-proj-abc123..." --key-type openai

# Test a Claude key
key-hunter test "sk-ant-..." --key-type claude
```

---

### Report Command

Create GitHub issues for exposed keys.

```bash
key-hunter report [OPTIONS]

Options:
  -r, --results-dir <DIR>      Directory containing result files [default: results]
  -k, --key-type <TYPE>        Filter by key type [default: all]
      --dry-run                Show what would be created without creating issues
```

**How it works:**
- Reads all JSON files in `results/` directory
- Deduplicates by repository (multiple keys = one issue per repo)
- Uses service-specific templates for each key type
- Creates issues with "security", "exposed-credentials", "urgent" labels

**Important:** Report processes ALL JSON files in the results directory. If you've already reported keys, either:
- Move processed files to an archive directory, or
- Use `--dry-run` to preview before creating issues

**Examples:**
```bash
# Preview issues without creating them
key-hunter report --dry-run

# Create issues for all validated keys
key-hunter report

# Only report Shodan keys
key-hunter report --key-type shodan

# Report from different directory
key-hunter report --results-dir old_scans/
```

---

### List Command

List available detectors and validators.

```bash
key-hunter list [what]

Arguments:
  what    What to list: detectors, validators, all [default: all]
```

**Examples:**
```bash
# List all detectors and validators
key-hunter list

# List only detectors
key-hunter list detectors

# List only validators
key-hunter list validators
```

---

## Output Structure

Results are automatically organized by key type with timestamps:

```
results/
├── shodan/
│   ├── valid_keys_20251110_143022.json
│   └── valid_keys_20251110_150000.json
├── openai/
│   └── valid_keys_20251110_151500.json
└── claude/
    └── valid_keys_20251110_160000.json
```

### Output Format

Each results file contains:

```json
{
  "timestamp": "2025-11-10T14:30:22Z",
  "key_type": "shodan",
  "total_valid_keys": 5,
  "total_keys_scanned": 12,
  "valid_keys": [
    {
      "detected": {
        "key": "ABC123...",
        "key_type": "shodan",
        "file_path": ".env",
        "line_number": 5,
        "repository": "user/repo",
        "file_url": "https://github.com/user/repo/blob/main/.env"
      },
      "validation": {
        "valid": true,
        "message": "Key is valid",
        "metadata": {
          "plan": "Free",
          "scan_credits": 100
        }
      },
      "validated_at": "2025-11-10T14:30:25Z"
    }
  ]
}
```

**Only valid keys are saved.** Invalid or expired keys are not written to disk.

---

## Configuration

### Environment Variables

Create a `.env` file in the project root:

```bash
# GitHub tokens for searching (1-5 tokens, auto-rotates when rate limited)
GITHUB_TOKEN1=ghp_your_first_token
GITHUB_TOKEN2=ghp_your_second_token  # Optional
GITHUB_TOKEN3=ghp_your_third_token   # Optional
GITHUB_TOKEN4=ghp_your_fourth_token  # Optional
GITHUB_TOKEN5=ghp_your_fifth_token   # Optional

# GitHub token for creating issues (separate account recommended)
ISSUES_GITHUB_TOKEN=ghp_issues_token
```

**Multi-token benefits:**
- Single token: ~30 file types/minute
- 5 tokens: ~150 file types/minute (5x throughput)
- Automatic rotation when rate limits are hit
- No manual intervention required

### Rate Limiting

**GitHub Code Search API:**
- **Authenticated:** 30 requests/minute per token
- **Unauthenticated:** 10 requests/minute per IP
- **Key Hunter delay:** 1000ms between GitHub search requests

**Validator Rate Limits (Configurable):**

Each API validator has configurable rate limits based on their API restrictions:

| Service | Default | RPM | Notes |
|---------|---------|-----|-------|
| OpenAI | 1000ms | 60 | Conservative for free tier |
| Claude | 2000ms | 30 | Matches Tier 1 limit |
| Gemini | 2000ms | 30 | Safe for paid tier |
| Shodan | 1000ms | 60 | Enforced 1 req/sec |
| xAI | 1000ms | 60 | Conservative default |
| OpenRouter | 3000ms | 20 | Free tier limit |
| GitHub | 2000ms | 30 | Secondary rate limit safe |

### Configuration File (Optional)

Create `config/default.toml` to customize validator rate limits and other settings:

```toml
[github]
base_url = "https://api.github.com"
rate_limit_delay_ms = 1000

[output]
format = "json"
directory = "./results"

# Validator rate limits (in milliseconds)
[validators]
openai_rate_limit_ms = 1000      # 60 RPM - conservative for free tier
claude_rate_limit_ms = 2000      # 30 RPM - matches Tier 1 limit
gemini_rate_limit_ms = 2000      # 30 RPM - safe for paid tier
shodan_rate_limit_ms = 1000      # 60 RPM - enforced 1 req/sec
xai_rate_limit_ms = 1000         # 60 RPM - conservative default
openrouter_rate_limit_ms = 3000  # 20 RPM - free tier limit
github_rate_limit_ms = 2000      # 30 RPM - secondary rate limit safe
```

**Customizing rate limits:**
- Increase values if you have higher tier API access
- Never lower below defaults unless you have confirmed higher limits
- Values are in milliseconds (1000ms = 1 second = 60 RPM)

---

## Workflows

### Basic Search & Validate Workflow

1. **Search for keys:**
   ```bash
   key-hunter search --key-type shodan --validate
   ```

2. **Review results:**
   ```bash
   cat results/shodan/valid_keys_*.json
   ```

3. **Create GitHub issues:**
   ```bash
   key-hunter report --key-type shodan --dry-run  # Preview first
   key-hunter report --key-type shodan            # Actually create
   ```

### Search Without Validation (Faster)

1. **Search only:**
   ```bash
   key-hunter search --key-type openai
   ```

2. **Validate later:**
   ```bash
   key-hunter validate -i results/openai/keys_*.json -o validated.json
   ```

3. **Report findings:**
   ```bash
   key-hunter report
   ```

### Multi-Service Scan

```bash
# Scan for all supported key types
key-hunter search --key-type all --validate

# Review all results
ls -lh results/*/

# Report all findings
key-hunter report --dry-run  # Preview
key-hunter report            # Create issues
```

---

## Tips

### Performance

- **Use multiple GitHub tokens** - Up to 5x faster with 5 tokens
- **Search without validation first** - Then validate in batch if you have many results
- **Use specific key types** - `--key-type shodan` is faster than `--key-type all`
- **Monitor with `--verbose`** - See exactly what's happening in real-time

### Best Practices

- **Always dry-run reports first** - Use `--dry-run` to preview before creating issues
- **Archive processed results** - Move reported files to prevent duplicate reports
- **Use separate tokens** - Different GitHub account for `ISSUES_GITHUB_TOKEN`
- **Respect rate limits** - Only adjust validator rate limits if you have confirmed higher API tier limits
- **Test single keys** - Use `test` command to debug validation issues
