use std::path::PathBuf;

use ts_rs::TS;

#[derive(TS)]
#[ts(export)]
pub struct ForwardLinksRequest {
    file_path: PathBuf,
}

#[derive(TS)]
#[ts(export)]
pub struct ForwardLinksResponse {}
