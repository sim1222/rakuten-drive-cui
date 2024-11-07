use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshTokenResponse {
    pub uid: String,
    pub email: String,
    pub email_verified: bool,
    pub display_name: String,
    pub disabled: bool,
    pub metadata: RefreshTokenResponseMetadata,
    pub provider_data: Vec<serde_json::Value>,
    pub custom_claims: RefreshTokenResponseCustomClaims,
    pub tokens_valid_after_time: String, // Wed, 17 Jul 2024 14:20:15 GMT
    pub refresh_token: String,
    pub id_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshTokenResponseMetadata {
    pub last_sign_in_time: String, // Wed, 17 Jul 2024 14:20:15 GMT
    pub creation_time: String,     // Wed, 17 Jul 2024 14:20:15 GMT
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenResponseCustomClaims {
    pub plan: String, // skf = 50GB free, sk3 = Unlimited
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ListFilesResponse {
    pub access_level: String,
    pub count: i64,
    pub file: Vec<ListFilesResponseFile>,
    pub last_page: bool,
    pub owner: String,
    pub prefix: String,
    pub usage_size: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListFilesResponseFile {
    pub has_child_folder: bool,
    pub is_backed_up: bool,
    pub is_folder: bool,
    pub is_latest: bool,
    pub is_share: String,
    pub last_modified: String, // 2024-07-16T06:18:06.595Z

    #[serde(rename = "OwnerID")]
    pub owner_id: String, // OwnerID
    pub path: String,
    pub size: i64,
    pub thumbnail: String,

    #[serde(rename = "VersionID")]
    pub version_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct JobKeyResponse {
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CheckActionResponse {
    pub action: String,
    pub state: String,
    pub usage_size: Option<i64>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CheckUploadResponse {
    pub bucket: String,
    pub file: Vec<CheckUploadResponseFile>,
    pub prefix: String,
    pub region: String,
    pub upload_id: String,
}

#[derive(Debug, Serialize, Deserialize)]

pub struct CheckUploadResponseFile {
    pub last_modified: String, // 1970-01-20T22:07:12.804Z
    pub path: String,
    pub size: i64,
    pub version_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetFileLinkTokenResponse {
    pub access_key_id: String,
    pub expiration: String, // 2024-07-16T06:18:06.595Z
    pub secret_access_key: String,
    pub session_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GetFileLinkResponse {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FileDetailResponse {
    pub access_level: String,
    pub file: FileDetailResponseFile,
    pub owner: String,
    pub prefix: String,
    pub usage_size: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FileDetailResponseFile {
    pub access_level: String,
    pub has_child_folder: bool,

    #[serde(rename = "HostID")]
    pub host_id: String,
    pub is_backed_up: bool,
    pub is_folder: bool,
    pub is_latest: bool,
    pub is_share: String,
    pub items_count: i64,
    pub last_modified: String, // 2024-07-16T06:18:06.595Z

    #[serde(rename = "LastModifierID")]
    pub last_modifier_id: String,

    #[serde(rename = "OwnerID")]
    pub owner_id: String, // OwnerID
    pub path: String,
    pub size: i64,
    pub thumbnail: String,
    pub version: serde_json::Value, // returns null

    #[serde(rename = "VersionID")]
    pub version_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SendyError {
    pub error: SendyErrorType,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SendyErrorType {
    SendyErrFileNoFolder,
    SendyErrFileNoSuchKey,
    SendyErrFileAlreadyExistFileName,
    SendyErrFileLongKey,
    SendyErrExceededFolderMaxStorage,
    SendyErrExceededTraffic,
    SendyErrServer,
    SendyErrAlreadyRunning,
    SendyErrNoLinkToSave,
    SendyErrPasswordNotMatch,
    SendyErrLinkExpired,
    SendyErrUninvitedUser,
    SendyErrFileWrongPath,
    SendyErrFileNoPermission,
    SendyErrLinkInvalidPassword,
    SendyErrShareDownwardShareExist,
    SendyErrShareUpwardShareExist,
    SendyErrShareFolderIncludedOrInclude,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RIDTokenResponse {
    pub custom_token: String,
    pub is_new_user: String, // "false"
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyCustomTokenResponse {
    pub kind: String,
    pub id_token: String,
    pub refresh_token: String,
    pub expires_in: String,
    pub is_new_user: bool,
}