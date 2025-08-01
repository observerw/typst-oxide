use crate::parser::models::Label;
use regex::Regex;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum LabelError {
    #[error("regex compilation failed")]
    RegexError(#[from] regex::Error),
}

pub struct LabelParser {
    explicit_label_regex: Regex,
    heading_regex: Regex,
}

impl LabelParser {
    pub fn new() -> Result<Self, LabelError> {
        // Matches explicit labels: <label-name>
        let explicit_label_regex = Regex::new(r"<([a-zA-Z0-9_:.-]+)>")?;

        // Matches headings: = Heading, == Subsection, etc.
        let heading_regex = Regex::new(r"^(=+)\s+(.+)$")?;

        Ok(Self {
            explicit_label_regex,
            heading_regex,
        })
    }

    pub fn parse_labels(&self, content: &str, file_path: &Path) -> Vec<Label> {
        let mut labels = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            // Parse explicit labels
            for cap in self.explicit_label_regex.captures_iter(line) {
                let label_name = cap.get(1).unwrap().as_str().to_string();
                let full_match = cap.get(0).unwrap();
                let column = line[..full_match.start()].chars().count() + 1;

                labels.push(Label {
                    name: label_name,
                    line: line_idx + 1,
                    column,
                    is_implicit: false,
                });
            }

            // Parse headings as implicit labels
            if let Some(cap) = self.heading_regex.captures(line) {
                let heading_text = cap.get(2).unwrap().as_str().trim();
                if !heading_text.is_empty() {
                    // Convert heading to label format: lowercase, replace spaces with hyphens
                    let label_name = heading_text
                        .to_lowercase()
                        .chars()
                        .map(|c| {
                            if c.is_alphanumeric() || c == '-' || c == '_' {
                                c
                            } else {
                                '-'
                            }
                        })
                        .collect::<String>()
                        .split('-')
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<_>>()
                        .join("-");

                    if !label_name.is_empty() {
                        labels.push(Label {
                            name: label_name,
                            line: line_idx + 1,
                            column: 1,
                            is_implicit: true,
                        });
                    }
                }
            }
        }

        labels
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_explicit_labels() {
        let parser = LabelParser::new().unwrap();
        let content = "Some text with <my-label> and <another_label>";
        let path = PathBuf::from("test.typ");

        let labels = parser.parse_labels(content, &path);

        assert_eq!(labels.len(), 2);
        assert_eq!(labels[0].name, "my-label");
        assert_eq!(labels[0].is_implicit, false);
        assert_eq!(labels[1].name, "another_label");
        assert_eq!(labels[1].is_implicit, false);
    }

    #[test]
    fn test_parse_headings_as_labels() {
        let parser = LabelParser::new().unwrap();
        let content = "= Main Heading\n== Subsection\n=== Deep Section";
        let path = PathBuf::from("test.typ");

        let labels = parser.parse_labels(content, &path);

        assert_eq!(labels.len(), 3);
        assert_eq!(labels[0].name, "main-heading");
        assert_eq!(labels[0].is_implicit, true);
        assert_eq!(labels[1].name, "subsection");
        assert_eq!(labels[1].is_implicit, true);
        assert_eq!(labels[2].name, "deep-section");
        assert_eq!(labels[2].is_implicit, true);
    }

    #[test]
    fn test_parse_mixed_labels() {
        let parser = LabelParser::new().unwrap();
        let content = "= Section\nSome text with <explicit-label>\n== Subsection";
        let path = PathBuf::from("test.typ");

        let labels = parser.parse_labels(content, &path);

        assert_eq!(labels.len(), 3);

        let explicit_labels: Vec<_> = labels.iter().filter(|l| !l.is_implicit).collect();
        let implicit_labels: Vec<_> = labels.iter().filter(|l| l.is_implicit).collect();

        assert_eq!(explicit_labels.len(), 1);
        assert_eq!(explicit_labels[0].name, "explicit-label");

        assert_eq!(implicit_labels.len(), 2);
        assert_eq!(implicit_labels[0].name, "section");
        assert_eq!(implicit_labels[1].name, "subsection");
    }

    #[test]
    fn test_heading_with_special_chars() {
        let parser = LabelParser::new().unwrap();
        let content = "= Hello World! This is a test.";
        let path = PathBuf::from("test.typ");

        let labels = parser.parse_labels(content, &path);

        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0].name, "hello-world-this-is-a-test");
        assert_eq!(labels[0].is_implicit, true);
    }

    #[test]
    fn test_empty_heading() {
        let parser = LabelParser::new().unwrap();
        let content = "=   ";
        let path = PathBuf::from("test.typ");

        let labels = parser.parse_labels(content, &path);

        assert!(labels.is_empty());
    }

    #[test]
    fn test_label_positions() {
        let parser = LabelParser::new().unwrap();
        let content = "    <label>\n= Heading";
        let path = PathBuf::from("test.typ");

        let labels = parser.parse_labels(content, &path);

        assert_eq!(labels.len(), 2);
        assert_eq!(labels[0].name, "label");
        assert_eq!(labels[0].line, 1);
        assert_eq!(labels[0].column, 5);

        assert_eq!(labels[1].name, "heading");
        assert_eq!(labels[1].line, 2);
        assert_eq!(labels[1].column, 1);
    }

    #[test]
    fn test_complex_label_names() {
        let parser = LabelParser::new().unwrap();
        let content = "<label:with:colons> and <label-with-dots...>";
        let path = PathBuf::from("test.typ");

        let labels = parser.parse_labels(content, &path);

        assert_eq!(labels.len(), 2);
        assert_eq!(labels[0].name, "label:with:colons");
        assert_eq!(labels[1].name, "label-with-dots...");
    }
}
