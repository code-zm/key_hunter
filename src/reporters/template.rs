use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Simple template renderer that replaces {{variable}} placeholders
pub struct TemplateRenderer {
    template: String,
    pub path: PathBuf,
}

impl TemplateRenderer {
    /// Load a template from a file
    /// First tries .html extension, then falls back to .md
    pub fn load(template_name: &str) -> Result<Self, std::io::Error> {
        // Try HTML first
        let html_path = Path::new("templates").join(format!("{}.html", template_name));
        if let Ok(template) = fs::read_to_string(&html_path) {
            return Ok(Self {
                template,
                path: html_path,
            });
        }

        // Fallback to markdown
        let md_path = Path::new("templates").join(format!("{}.md", template_name));
        let template = fs::read_to_string(&md_path)?;
        Ok(Self {
            template,
            path: md_path,
        })
    }

    /// Render the template with the given variables
    pub fn render(&self, variables: &HashMap<String, String>) -> String {
        let mut result = self.template.clone();

        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_rendering() {
        let template = TemplateRenderer {
            template: "Hello {{name}}, you are {{age}} years old.".to_string(),
            path: PathBuf::from("test.md"),
        };

        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Alice".to_string());
        vars.insert("age".to_string(), "30".to_string());

        let result = template.render(&vars);
        assert_eq!(result, "Hello Alice, you are 30 years old.");
    }
}
