use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ParsedFile {
    pub path: PathBuf,
    pub metadata: Metadata,
    pub wikilinks: Vec<Wikilink>,
    pub labels: Vec<Label>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Metadata {
    pub title: Option<String>,
    pub tags: Vec<String>,
    pub alias: Vec<String>,
    #[ts(type = "Record<string, any>")]
    pub custom: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Wikilink {
    pub target: String,
    pub alias: Option<String>,
    pub label: Option<String>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Label {
    pub name: String,
    pub line: usize,
    pub column: usize,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            title: None,
            tags: Vec::new(),
            alias: Vec::new(),
            custom: HashMap::new(),
        }
    }
}
