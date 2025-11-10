## Security Alert: Exposed API Key Detected

An automated security scan has detected a **valid {{service_name}} API key** that is publicly exposed in this repository.

### Details

- **Service**: {{service_name}}
- **File**: `{{file_path}}`
- **Line Number**: {{line_number}}
- **File URL**: {{file_url}}
- **Key Preview**: `{{key_partial}}` (truncated for security)

### Validation Results

This key was **validated as active** against the {{service_name}} API:

{{metadata_section}}

### Immediate Actions Required

1. **REVOKE THIS KEY IMMEDIATELY** at {{revoke_url}}
2. **Generate a new API key** with appropriate permissions
3. **Remove the key from this file** and all git history
4. **Use environment variables or secret management** instead of hardcoding keys
5. **Rotate any other credentials** that may have been exposed{{additional_actions}}

### Remediation Steps

**Remove from Git History:**
```bash
# Using git-filter-repo (recommended)
git filter-repo --path {{file_path}} --invert-paths

# Or using BFG Repo-Cleaner
bfg --delete-files {{file_path}}
git reflog expire --expire=now --all && git gc --prune=now --aggressive
```

**Secure Storage Best Practices:**
- Store keys in `.env` files (add to `.gitignore`)
- Use secret management services (AWS Secrets Manager, HashiCorp Vault, etc.)
- Use environment variables in CI/CD pipelines
- Never commit credentials to version control{{best_practices}}

### Resources

{{resources}}
- [GitHub: Removing sensitive data](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/removing-sensitive-data-from-a-repository)
- [OWASP: Password Storage Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html)

---

*This issue was created automatically by an API key scanner. Reported on {{timestamp}} UTC*
