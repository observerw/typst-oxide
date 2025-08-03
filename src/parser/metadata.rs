use crate::parser::models::Metadata;
use anyhow::Result;
use serde_json::Value;
use std::path::Path;
use tokio::process::Command;

pub async fn extract_metadata(file_path: &Path) -> Result<Metadata> {
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
                anyhow::anyhow!("typst command not found")
            } else {
                anyhow::anyhow!("failed to execute typst query: {}", e)
            }
        })?;

    if !output.status.success() {
        return Ok(Metadata::default());
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    parse_metadata_json(&json_str)
}

fn parse_metadata_json(json_str: &str) -> Result<Metadata> {
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
