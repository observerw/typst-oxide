use crate::parser::models::Metadata;
use serde_json::Value;
use std::path::Path;
use tokio::process::Command;

#[derive(Debug, thiserror::Error)]
pub enum MetadataError {
    #[error("typst command not found")]
    TypstNotFound,
    #[error("failed to execute typst query: {0}")]
    ExecutionError(String),
    #[error("invalid JSON from typst query: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("invalid metadata format")]
    InvalidFormat,
}

pub async fn extract_metadata(file_path: &Path) -> Result<Metadata, MetadataError> {
    let output = Command::new("typst")
        .arg("query")
        .arg(file_path)
        .arg("metadata")
        .arg("--field")
        .arg("value")
        .arg("--one")
        .output()
        .await
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                MetadataError::TypstNotFound
            } else {
                MetadataError::ExecutionError(e.to_string())
            }
        })?;

    if !output.status.success() {
        return Ok(Metadata::default());
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    parse_metadata_json(&json_str)
}

fn parse_metadata_json(json_str: &str) -> Result<Metadata, MetadataError> {
    let value: Value = serde_json::from_str(json_str)?;
    
    let mut metadata = Metadata::default();
    
    if let Some(obj) = value.as_object() {
        for (key, val) in obj {
            match key.as_str() {
                "title" => {
                    metadata.title = val.as_str().map(String::from);
                }
                "tags" => {
                    if let Some(arr) = val.as_array() {
                        metadata.tags = arr
                            .iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect();
                    }
                }
                "alias" => {
                    if let Some(arr) = val.as_array() {
                        metadata.alias = arr
                            .iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect();
                    }
                }
                _ => {
                    metadata.custom.insert(key.clone(), val.clone());
                }
            }
        }
    }
    
    Ok(metadata)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_metadata_json() {
        let json = r#"{
            "title": "Test Note",
            "tags": ["tag1", "tag2"],
            "alias": ["alias1"],
            "custom_field": "value"
        }"#;
        
        let metadata = parse_metadata_json(json).unwrap();
        
        assert_eq!(metadata.title, Some("Test Note".to_string()));
        assert_eq!(metadata.tags, vec!["tag1", "tag2"]);
        assert_eq!(metadata.alias, vec!["alias1"]);
        assert_eq!(metadata.custom.get("custom_field"), Some(&json!("value")));
    }

    #[test]
    fn test_parse_empty_metadata() {
        let json = "{}";
        let metadata = parse_metadata_json(json).unwrap();
        
        assert_eq!(metadata.title, None);
        assert!(metadata.tags.is_empty());
        assert!(metadata.alias.is_empty());
    }

    #[test]
    fn test_parse_invalid_json() {
        let json = "invalid json";
        let result = parse_metadata_json(json);
        assert!(result.is_err());
    }
}