use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Simple template renderer that replaces {{variable}} placeholders
pub struct TemplateRenderer {
    template: String,
}

impl TemplateRenderer {
    /// Load a template from a file
    pub fn load(template_name: &str) -> Result<Self, std::io::Error> {
        let template_path = Path::new("templates").join(format!("{}.md", template_name));
        let template = fs::read_to_string(template_path)?;
        Ok(Self { template })
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
        };

        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Alice".to_string());
        vars.insert("age".to_string(), "30".to_string());

        let result = template.render(&vars);
        assert_eq!(result, "Hello Alice, you are 30 years old.");
    }
}
