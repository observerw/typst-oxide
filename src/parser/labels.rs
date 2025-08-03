use crate::parser::models::Label;
use anyhow::Result;
use regex::Regex;

pub struct LabelParser {
    label_regex: Regex,
    heading_regex: Regex,
}

impl LabelParser {
    pub fn new() -> Result<Self> {
        // Matches explicit labels: <label-name>
        let explicit_label_regex = Regex::new(r"<([a-zA-Z0-9_:.-]+)>")?;

        // Matches headings: = Heading, == Subsection, etc.
        let heading_regex = Regex::new(r"^(=+)\s+(.+)$")?;

        Ok(Self {
            label_regex: explicit_label_regex,
            heading_regex,
        })
    }

    pub fn parse_labels(&self, content: &str) -> Vec<Label> {
        let mut labels = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            // Parse explicit labels
            for cap in self.label_regex.captures_iter(line) {
                let label_name = cap.get(1).unwrap().as_str().to_string();
                let full_match = cap.get(0).unwrap();
                let column = line[..full_match.start()].chars().count() + 1;

                labels.push(Label {
                    name: label_name,
                    line: line_idx + 1,
                    column,
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
                        });
                    }
                }
            }
        }

        labels
    }
}
