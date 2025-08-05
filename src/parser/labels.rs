use crate::parser::models::Label;
use anyhow::Result;
use regex::Regex;

pub struct LabelParser {
    label_regex: Regex,
}

impl LabelParser {
    pub fn new() -> Result<Self> {
        // Matches explicit labels: <label-name>
        let explicit_label_regex = Regex::new(r"<([a-zA-Z0-9_:.-]+)>")?;

        Ok(Self {
            label_regex: explicit_label_regex,
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


        }

        labels
    }
}
