use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use ts_rs::TS;

use crate::{index::Index, parser::models::Wikilink};

// Forward Links Commands
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ForwardLinksRequest {
    pub file_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ForwardLinksResponse {
    pub links: Vec<Wikilink>,
}

// Backward Links Commands
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BackwardLinksRequest {
    pub file_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BackwardLinksResponse {
    pub links: Vec<BacklinkInfo>,
}

// Helper struct for backward links that includes source file information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BacklinkInfo {
    pub source_file: PathBuf,
    pub wikilink: Wikilink,
}

// Command handlers that wrap index module functionality
use anyhow::Result;

/// Process forward links request by wrapping Index::get_forward_links
pub fn handle_forward_links(
    index: &Index,
    request: ForwardLinksRequest,
) -> Result<ForwardLinksResponse> {
    let links = index.get_forward_links(&request.file_path)?;
    Ok(ForwardLinksResponse { links })
}

/// Process backward links request by wrapping Index::get_backward_links
pub fn handle_backward_links(
    index: &Index,
    request: BackwardLinksRequest,
) -> Result<BackwardLinksResponse> {
    let backlinks = index.get_backward_links(&request.file_path)?;
    let links = backlinks
        .into_iter()
        .map(|(source_file, wikilink)| BacklinkInfo {
            source_file,
            wikilink,
        })
        .collect();
    Ok(BackwardLinksResponse { links })
}
