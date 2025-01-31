use std::{collections::HashMap, sync::Arc};

use crate::{
    common::state::Status,
    eventstream::service::MountSubscription,
    repo_files::{errors::LoadFilesError, state::RepoFile},
    repo_files_read::errors::GetFilesReaderError,
};

pub struct RepoFilesDetailsInfo<'a> {
    pub repo_id: Option<&'a str>,
    pub parent_path: Option<&'a str>,
    pub path: Option<&'a str>,
    pub status: Status<LoadFilesError>,
    pub file: Option<&'a RepoFile>,
    pub content_status: Status<GetFilesReaderError>,
    pub can_download: bool,
    pub can_copy: bool,
    pub can_move: bool,
    pub can_delete: bool,
}

#[derive(Clone)]
pub struct RepoFilesDetailsContent {
    pub status: Status<GetFilesReaderError>,
    pub bytes: Option<Vec<u8>>,
    pub version: u32,
}

#[derive(Clone)]
pub struct RepoFilesDetailsLocation {
    pub repo_id: String,
    pub path: String,
    pub eventstream_mount_subscription: Option<Arc<MountSubscription>>,
    pub content: RepoFilesDetailsContent,
}

#[derive(Clone)]
pub struct RepoFilesDetails {
    pub location: Option<RepoFilesDetailsLocation>,
    pub status: Status<LoadFilesError>,
}

#[derive(Clone, Default)]
pub struct RepoFilesDetailsState {
    pub details: HashMap<u32, RepoFilesDetails>,
    pub next_id: u32,
}
