## 🔐 Security Alert: Exposed API Key{{key_count_plural}} Detected

Hello,

An automated security scan has detected **{{key_count}} valid {{service_name}} API key{{key_count_plural}}** that {{key_count_verb}} publicly exposed in your repository: **{{repository}}**

This is a courtesy notification to help you secure your credentials.

### Details

{{keys_details}}

### Validation Results

{{key_count_these}} key{{key_count_plural}} {{key_count_verb_past}} **validated as active** against the {{service_name}} API:

{{metadata_section}}

### Immediate Actions Required

1. **REVOKE {{key_count_these_upper}} KEY{{key_count_plural_upper}} IMMEDIATELY** at {{revoke_url}}
2. **Generate new API key{{key_count_plural}}** with appropriate permissions
3. **Remove {{key_count_the}} key{{key_count_plural}} from {{key_count_these}} file{{key_count_plural}}** and all git history
4. **Use environment variables or secret management** instead of hardcoding keys
5. **Rotate any other credentials** that may have been exposed{{additional_actions}}

### Remediation Steps

**Remove from Git History:**
```bash
# Using git-filter-repo (recommended)
{{file_cleanup_commands}}

# Or clean entire history with BFG Repo-Cleaner
bfg --delete-files "*.env"
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

### Why did I receive this email?

You received this notification because you are either:
- The owner of the repository **{{repository}}**
- The author of a commit containing an exposed API key

This notification was sent automatically by [key_hunter](https://github.com/code-zm/key_hunter), an open-source security tool designed to help developers secure their repositories.

**Reported on {{timestamp}} UTC**

If this report helped you secure your repository, please consider giving us a ⭐ on [GitHub](https://github.com/code-zm/key_hunter)!

---

*Please do not reply to this email. This is an automated security notification.*
