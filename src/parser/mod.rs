pub mod metadata;
pub mod wikilinks;
pub mod labels;
pub mod models;

use crate::parser::{
    labels::LabelParser,
    metadata::{extract_metadata, MetadataError},
    models::{Label, ParsedFile, Wikilink},
    wikilinks::WikilinkParser,
};
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("metadata extraction failed: {0}")]
    MetadataError(#[from] MetadataError),
    #[error("wikilink parsing failed: {0}")]
    WikilinkError(#[from] wikilinks::WikilinkError),
    #[error("label parsing failed: {0}")]
    LabelError(#[from] labels::LabelError),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub struct Parser {
    wikilink_parser: WikilinkParser,
    label_parser: LabelParser,
}

impl Parser {
    pub fn new() -> Result<Self, ParseError> {
        Ok(Self {
            wikilink_parser: WikilinkParser::new()?,
            label_parser: LabelParser::new()?,
        })
    }

    pub async fn parse_file(&self, file_path: &Path) -> Result<ParsedFile, ParseError> {
        let content = tokio::fs::read_to_string(file_path).await?;
        
        let metadata = extract_metadata(file_path).await?;
        let wikilinks = self.wikilink_parser.parse_wikilinks(&content, file_path);
        let labels = self.label_parser.parse_labels(&content, file_path);

        Ok(ParsedFile {
            path: file_path.to_path_buf(),
            metadata,
            wikilinks,
            labels,
        })
    }

    pub fn parse_content(&self, content: &str, file_path: &Path) -> Result<ParsedFile, ParseError> {
        let metadata = crate::parser::models::Metadata::default();
        let wikilinks = self.wikilink_parser.parse_wikilinks(content, file_path);
        let labels = self.label_parser.parse_labels(content, file_path);

        Ok(ParsedFile {
            path: file_path.to_path_buf(),
            metadata,
            wikilinks,
            labels,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_parse_complete_file() {
        let content = r#"#meta(
    title: "Test Note",
    tags: ("test", "example"),
    alias: ("alias1", "alias2")
)

= Main Section

This is a [[wikilink]] and [[another|with alias]].

== Subsection

See the section on [[file:label|Section Name]].

Here is an explicit label: <my-label>

Some math: $e^{i\pi} + 1 = 0$ <math>
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", content).unwrap();
        let file_path = temp_file.path();

        let parser = Parser::new().unwrap();
        let parsed = parser.parse_content(content, file_path).unwrap();

        // Check wikilinks
        assert_eq!(parsed.wikilinks.len(), 3);
        assert_eq!(parsed.wikilinks[0].target, "wikilink");
        assert_eq!(parsed.wikilinks[1].target, "another");
        assert_eq!(parsed.wikilinks[1].alias, Some("with alias".to_string()));
        assert_eq!(parsed.wikilinks[2].target, "file");
        assert_eq!(parsed.wikilinks[2].label, Some("label".to_string()));
        assert_eq!(parsed.wikilinks[2].alias, Some("Section Name".to_string()));

        // Check labels
        let explicit_labels: Vec<_> = parsed.labels.iter().filter(|l| !l.is_implicit).collect();
        let implicit_labels: Vec<_> = parsed.labels.iter().filter(|l| l.is_implicit).collect();

        assert_eq!(explicit_labels.len(), 2);
        assert_eq!(explicit_labels[0].name, "my-label");
        assert_eq!(explicit_labels[1].name, "math");

        assert_eq!(implicit_labels.len(), 2);
        assert_eq!(implicit_labels[0].name, "main-section");
        assert_eq!(implicit_labels[1].name, "subsection");
    }

    #[test]
    fn test_parse_empty_content() {
        let content = "";
        let file_path = Path::new("empty.typ");
        
        let parser = Parser::new().unwrap();
        let parsed = parser.parse_content(content, file_path).unwrap();

        assert!(parsed.wikilinks.is_empty());
        assert!(parsed.labels.is_empty());
    }

    #[test]
    fn test_parse_content_with_special_characters() {
        let content = r#"= Complex Heading: With Special-Chars!

Link to [[file-name.typ]] and [[file_name|alias-with-chars]].

Label: <label-with-special:chars...>
"#;

        let file_path = Path::new("special.typ");
        let parser = Parser::new().unwrap();
        let parsed = parser.parse_content(content, file_path).unwrap();

        assert_eq!(parsed.wikilinks.len(), 2);
        assert_eq!(parsed.wikilinks[0].target, "file-name.typ");
        assert_eq!(parsed.wikilinks[1].target, "file_name");

        assert_eq!(parsed.labels.len(), 2);
        assert_eq!(parsed.labels[0].name, "complex-heading-with-special-chars");
        assert_eq!(parsed.labels[0].is_implicit, true);
        assert_eq!(parsed.labels[1].name, "label-with-special:chars...");
        assert_eq!(parsed.labels[1].is_implicit, false);
    }
}