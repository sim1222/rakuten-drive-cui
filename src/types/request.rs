use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ListFilesRequest {
    pub from: i64,
    pub host_id: String,
    pub path: String,
    pub reverse: bool,
    pub sort_type: ListFilesRequestSortType,
    pub thumbnail_size: i64,
    pub to: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ListFilesRequestSortType {
    #[serde(rename = "name")]
    Path,
    #[serde(rename = "modified")]
    Modified,
    #[serde(rename = "size")]
    Size,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CreateFolderRequest {
    pub host_id: String,
    pub name: String,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RenameFileRequest {
    pub file: FileModifyRequestFile,
    pub host_id: String,
    pub name: String,
    pub prefix: String,
}

#[derive(Debug, Serialize, Deserialize)]

pub struct FileModifyRequestFile {
    pub last_modified: String, // 1970-01-20T22:07:12.804Z
    pub path: String,
    pub size: i64,
    pub version_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CheckActionRequest {
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MoveFileRequest {
    pub file: Vec<FileModifyRequestFile>,
    pub host_id: String,
    pub prefix: String,
    pub target_id: String,
    pub to_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CheckUploadRequest {
    pub file: Vec<CheckUploadRequestFile>,
    pub host_id: String,
    pub path: String,
    pub upload_id: String,
}

#[derive(Debug, Serialize, Deserialize)]

pub struct CheckUploadRequestFile {
    pub path: String,
    pub size: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CompleteUploadRequest {
    pub file: Vec<CompleteUploadRequestFile>,
    pub host_id: String,
    pub path: String,
    pub state: String,
}

#[derive(Debug, Serialize, Deserialize)]

pub struct CompleteUploadRequestFile {
    pub path: String,
    pub size: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DeleteFileRequest {
    pub file: Vec<FileModifyRequestFile>,
    pub host_id: String,
    pub prefix: String,
    pub trash: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GetFileLinkRequest {
    pub app_version: String,
    pub file: Vec<GetFileLinkRequestFile>,
    pub host_id: String,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetFileLinkRequestFile {
    pub path: String,
    pub size: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FileDetailRequest {
    pub host_id: String,
    pub path: String,
    pub thumbnail_size: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CopyFileRequest {
    pub file: Vec<CopyFileRequestFile>,
    pub host_id: String,
    pub prefix: String,
    pub target_id: String,
    pub to_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CopyFileRequestFile {
    pub last_modified: String, // 1970-01-20T22:07:12.804Z
    pub path: String,
    pub size: i64,
    pub version_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyCustomTokenRequest {
    pub return_secure_token: bool,
    pub token: String,
}