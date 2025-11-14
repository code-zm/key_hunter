use crate::core::error::{KeyHunterError, Result};
use crate::core::ValidatedKey;
use crate::reporters::template::TemplateRenderer;
use lettre::message::{header, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use std::collections::HashMap;
use tracing::{info, warn};

/// SMTP configuration for sending emails
#[derive(Clone, Debug)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_email: String,
    pub from_name: String,
}

impl SmtpConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            host: std::env::var("SMTP_HOST")
                .map_err(|_| KeyHunterError::Config("SMTP_HOST not set".to_string()))?,
            port: std::env::var("SMTP_PORT")
                .unwrap_or_else(|_| "587".to_string())
                .parse()
                .map_err(|_| KeyHunterError::Config("Invalid SMTP_PORT".to_string()))?,
            username: std::env::var("SMTP_USERNAME")
                .map_err(|_| KeyHunterError::Config("SMTP_USERNAME not set".to_string()))?,
            password: std::env::var("SMTP_PASSWORD")
                .map_err(|_| KeyHunterError::Config("SMTP_PASSWORD not set".to_string()))?,
            from_email: std::env::var("SMTP_FROM_EMAIL")
                .map_err(|_| KeyHunterError::Config("SMTP_FROM_EMAIL not set".to_string()))?,
            from_name: std::env::var("SMTP_FROM_NAME")
                .unwrap_or_else(|_| "Key Hunter Security Alert".to_string()),
        })
    }
}

/// Email client for sending security notifications
pub struct EmailClient {
    config: SmtpConfig,
    mailer: SmtpTransport,
}

impl EmailClient {
    pub fn new(config: SmtpConfig) -> Result<Self> {
        let creds = Credentials::new(config.username.clone(), config.password.clone());

        let mailer = SmtpTransport::starttls_relay(&config.host)
            .map_err(|e| KeyHunterError::Unknown(format!("SMTP connection error: {}", e)))?
            .port(config.port)
            .credentials(creds)
            .build();

        Ok(Self { config, mailer })
    }

    /// Send email notification to repository/commit author
    pub fn send_notification(
        &self,
        recipient_email: &str,
        recipient_name: Option<&str>,
        keys: &[ValidatedKey],
    ) -> Result<()> {
        if keys.is_empty() {
            return Ok(());
        }

        let repository = &keys[0].detected.repository;
        let key_type = &keys[0].detected.key_type;

        // Build email subject
        let subject = if keys.len() == 1 {
            format!(
                "[Security Alert] Exposed {} API Key in {}",
                key_type.to_uppercase(),
                repository
            )
        } else {
            format!(
                "[Security Alert] {} Exposed API Keys in {}",
                keys.len(),
                repository
            )
        };

        // Render email body
        let (html_body, text_body) = self.render_email_body(keys, repository)?;

        // Build recipient
        let to_address = if let Some(name) = recipient_name {
            format!("{} <{}>", name, recipient_email)
        } else {
            recipient_email.to_string()
        };

        // Build email
        let email = Message::builder()
            .from(
                format!("{} <{}>", self.config.from_name, self.config.from_email)
                    .parse()
                    .map_err(|e| KeyHunterError::Unknown(format!("Invalid from address: {}", e)))?,
            )
            .to(to_address
                .parse()
                .map_err(|e| KeyHunterError::Unknown(format!("Invalid to address: {}", e)))?)
            .subject(subject)
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::TEXT_PLAIN)
                            .body(text_body),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::TEXT_HTML)
                            .body(html_body),
                    ),
            )
            .map_err(|e| KeyHunterError::Unknown(format!("Failed to build email: {}", e)))?;

        // Send email
        match self.mailer.send(&email) {
            Ok(_) => {
                info!("Email sent successfully to {}", recipient_email);
                Ok(())
            }
            Err(e) => {
                warn!("Failed to send email to {}: {}", recipient_email, e);
                Err(KeyHunterError::Unknown(format!(
                    "Failed to send email: {}",
                    e
                )))
            }
        }
    }

    /// Render email body as both HTML and plain text
    fn render_email_body(
        &self,
        keys: &[ValidatedKey],
        repository: &str,
    ) -> Result<(String, String)> {
        let template = TemplateRenderer::load("email")
            .or_else(|_| TemplateRenderer::load("issue"))?;

        let key_count = keys.len();
        let key_type = &keys[0].detected.key_type;

        // Build variables for template
        let mut vars = HashMap::new();
        vars.insert("repository".to_string(), repository.to_string());
        vars.insert("key_count".to_string(), key_count.to_string());

        // Service-specific configuration
        let service_config = crate::reporters::ServiceConfig::get(key_type);

        vars.insert("service_name".to_string(), service_config.service_name);

        // Build keys details section
        let mut keys_details = Vec::new();
        let mut keys_details_html = Vec::new();
        let mut metadata_sections = Vec::new();
        let mut metadata_sections_html = Vec::new();
        let mut file_cleanup_commands = Vec::new();

        for (idx, validated_key) in keys.iter().enumerate() {
            let detected = &validated_key.detected;

            // Key details
            let key_preview = if detected.key.len() > 20 {
                format!("{}...", &detected.key[..20])
            } else {
                detected.key.clone()
            };

            // Markdown version
            keys_details.push(format!(
                "**Key {}:**\n- File: `{}`\n- Line: {}\n- URL: {}\n- Key: `{}`\n- Commit: {}",
                idx + 1,
                detected.file_path,
                detected
                    .line_number
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "N/A".to_string()),
                detected.file_url,
                key_preview,
                detected
                    .commit_sha
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string())
            ));

            // HTML version
            keys_details_html.push(format!(
                r#"<div style="margin-bottom: 20px; padding-bottom: 20px; border-bottom: 1px solid #e5e7eb;">
                    <div style="font-weight: 600; color: #dc2626; margin-bottom: 10px; font-size: 15px;">Key {}:</div>
                    <table style="width: 100%; font-size: 14px; color: #374151;">
                        <tr><td style="padding: 4px 0; width: 80px; vertical-align: top;"><strong>File:</strong></td><td style="padding: 4px 0;"><code style="background-color: #f3f4f6; padding: 2px 6px; border-radius: 3px; font-family: 'Courier New', monospace; font-size: 12px; word-break: break-all;">{}</code></td></tr>
                        <tr><td style="padding: 4px 0; vertical-align: top;"><strong>Line:</strong></td><td style="padding: 4px 0;">{}</td></tr>
                        <tr><td style="padding: 4px 0; vertical-align: top;"><strong>URL:</strong></td><td style="padding: 4px 0;"><a href="{}" style="color: #2563eb; text-decoration: none; word-break: break-all;">{}</a></td></tr>
                        <tr><td style="padding: 4px 0; vertical-align: top;"><strong>Key:</strong></td><td style="padding: 4px 0;"><code style="background-color: #fef2f2; color: #dc2626; padding: 2px 6px; border-radius: 3px; font-family: 'Courier New', monospace; font-size: 12px;">{}</code></td></tr>
                        <tr><td style="padding: 4px 0; vertical-align: top;"><strong>Commit:</strong></td><td style="padding: 4px 0;"><code style="background-color: #f3f4f6; padding: 2px 6px; border-radius: 3px; font-family: 'Courier New', monospace; font-size: 12px;">{}</code></td></tr>
                    </table>
                </div>"#,
                idx + 1,
                detected.file_path,
                detected
                    .line_number
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "N/A".to_string()),
                detected.file_url,
                detected.file_url,
                key_preview,
                detected
                    .commit_sha
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string())
            ));

            // Metadata
            let mut metadata_lines = Vec::new();
            let mut metadata_html_lines = Vec::new();
            for (key, value) in &validated_key.validation.metadata {
                let formatted_key = key
                    .split('_')
                    .map(|word| {
                        let mut chars = word.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(first) => first.to_uppercase().chain(chars).collect(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ");

                metadata_lines.push(format!("- **{}**: {}", formatted_key, value));
                metadata_html_lines.push(format!(
                    r#"<div style="margin-bottom: 8px; color: #374151; font-size: 14px;"><strong style="color: #059669;">{}:</strong> {}</div>"#,
                    formatted_key, value
                ));
            }
            metadata_sections.push(metadata_lines.join("\n"));
            metadata_sections_html.push(metadata_html_lines.join("\n"));

            // Cleanup command
            file_cleanup_commands.push(format!(
                "git filter-repo --path {} --invert-paths",
                detected.file_path
            ));
        }

        vars.insert("keys_details".to_string(), keys_details.join("\n\n"));
        vars.insert("keys_details_html".to_string(), keys_details_html.join("\n"));
        vars.insert(
            "metadata_section".to_string(),
            metadata_sections.join("\n\n"),
        );
        vars.insert(
            "metadata_section_html".to_string(),
            metadata_sections_html.join("\n<div style=\"margin: 15px 0; border-top: 1px solid #e5e7eb;\"></div>\n"),
        );
        vars.insert(
            "file_cleanup_commands".to_string(),
            file_cleanup_commands.join("\n"),
        );

        vars.insert(
            "timestamp".to_string(),
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        );

        // Render the template
        let rendered = template.render(&vars);

        // Determine if we're using HTML or markdown template based on file extension
        let (html_body, text_body) = if template.path.extension().and_then(|s| s.to_str()) == Some("html") {
            // HTML template - use as-is for HTML, generate plain text version
            let text_body = self.html_to_plain_text(&rendered);
            (rendered, text_body)
        } else {
            // Markdown template - wrap in basic HTML for HTML version
            let html_body = format!(
                "<html><body><pre style='font-family: monospace; white-space: pre-wrap;'>{}</pre></body></html>",
                rendered
            );
            (html_body, rendered)
        };

        Ok((html_body, text_body))
    }

    /// Convert HTML to plain text by stripping tags and entities
    fn html_to_plain_text(&self, html: &str) -> String {
        let mut text = html.to_string();

        // Remove script and style tags with their content
        let re_script = regex::Regex::new(r"<script[^>]*>.*?</script>").unwrap();
        let re_style = regex::Regex::new(r"<style[^>]*>.*?</style>").unwrap();
        text = re_script.replace_all(&text, "").to_string();
        text = re_style.replace_all(&text, "").to_string();

        // Replace common block elements with newlines
        let block_tags = ["</div>", "</p>", "</h1>", "</h2>", "</h3>", "</h4>", "</h5>", "</h6>", "<br>", "<br/>", "</tr>", "</li>"];
        for tag in &block_tags {
            text = text.replace(tag, &format!("{}\n", tag));
        }

        // Remove all HTML tags
        let re_tags = regex::Regex::new(r"<[^>]+>").unwrap();
        text = re_tags.replace_all(&text, "").to_string();

        // Decode common HTML entities
        text = text
            .replace("&nbsp;", " ")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&")
            .replace("&quot;", "\"")
            .replace("&#39;", "'");

        // Clean up excessive whitespace
        let re_whitespace = regex::Regex::new(r"\n{3,}").unwrap();
        text = re_whitespace.replace_all(&text, "\n\n").to_string();

        text.trim().to_string()
    }

    /// Send notifications in bulk, grouped by recipient
    pub fn send_bulk_notifications(&self, validated_keys: &[ValidatedKey]) -> Result<()> {
        // Group keys by recipient email
        let mut by_email: HashMap<String, Vec<ValidatedKey>> = HashMap::new();

        for key in validated_keys {
            // Prefer commit author email, fall back to repo owner email
            let email = key
                .detected
                .commit_author_email
                .as_ref()
                .or(key.detected.repo_owner_email.as_ref());

            if let Some(email_addr) = email {
                by_email
                    .entry(email_addr.clone())
                    .or_insert_with(Vec::new)
                    .push(key.clone());
            } else {
                warn!(
                    "No email found for key in repository {}",
                    key.detected.repository
                );
            }
        }

        info!(
            "Sending emails to {} recipient(s) for {} key(s)",
            by_email.len(),
            validated_keys.len()
        );

        // Send one email per recipient
        for (email, keys) in by_email {
            self.send_notification(&email, None, &keys)?;
        }

        Ok(())
    }
}
