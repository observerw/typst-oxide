pub mod labels;
pub mod metadata;
pub mod models;
pub mod wikilinks;

use crate::parser::{
    labels::LabelParser, metadata::extract_metadata, models::ParsedFile, wikilinks::WikilinkParser,
};
use anyhow::Result;
use std::path::Path;

pub type ParseError = anyhow::Error;

pub struct Parser {
    wikilink_parser: WikilinkParser,
    label_parser: LabelParser,
}

impl Parser {
    pub fn new() -> Result<Self> {
        Ok(Self {
            wikilink_parser: WikilinkParser::new()?,
            label_parser: LabelParser::new()?,
        })
    }

    pub async fn parse_file(&self, file_path: &Path) -> Result<ParsedFile> {
        let content = tokio::fs::read_to_string(file_path).await?;

        let metadata = extract_metadata(file_path).await?;
        let wikilinks = self.wikilink_parser.parse_wikilinks(&content, file_path);
        let labels = self.label_parser.parse_labels(&content);

        Ok(ParsedFile {
            path: file_path.to_path_buf(),
            metadata,
            wikilinks,
            labels,
        })
    }

    pub fn parse_content(&self, content: &str, file_path: &Path) -> Result<ParsedFile> {
        let metadata = crate::parser::models::Metadata::default();
        let wikilinks = self.wikilink_parser.parse_wikilinks(content, file_path);
        let labels = self.label_parser.parse_labels(content);

        Ok(ParsedFile {
            path: file_path.to_path_buf(),
            metadata,
            wikilinks,
            labels,
        })
    }
}
