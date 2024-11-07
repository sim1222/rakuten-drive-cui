use anyhow::Ok;
use reqwest::StatusCode;
use tokio::sync::RwLock;

use crate::types;

pub struct Client {
    pub token: RwLock<String>,
    pub refresh_token: String,
    pub last_refresh: RwLock<std::time::Instant>,
    pub host_id: String,
    token_valid_time: u32,
}

impl Client {
    pub async fn try_new(refresh_token_str: String) -> anyhow::Result<Self> {
        let refresh_token_req = types::request::RefreshTokenRequest {
            refresh_token: refresh_token_str,
        };
        let token = refresh_token(refresh_token_req).await.unwrap();
        Ok(Self {
            refresh_token: token.refresh_token,
            token: RwLock::new(token.id_token),
            last_refresh: RwLock::new(std::time::Instant::now()),
            host_id: token.uid,
            token_valid_time: 3600,
        })
    }
    pub async fn refresh_token(&self) -> anyhow::Result<()> {
        let refresh_token_req = types::request::RefreshTokenRequest {
            refresh_token: self.refresh_token.clone(),
        };
        let token = refresh_token(refresh_token_req).await.unwrap();
        *self.token.write().await = token.id_token;
        *self.last_refresh.write().await = std::time::Instant::now();
        Ok(())
    }
    pub async fn list_files(
        &self,
        req: types::request::ListFilesRequest,
    ) -> anyhow::Result<types::response::ListFilesResponse> {
        if self.last_refresh.read().await.elapsed().as_secs() > self.token_valid_time.into() {
            self.refresh_token().await?;
        }
        let client = reqwest::Client::new();
        let request = client
            .post("https://forest.sendy.jp/cloud/service/file/v1/files")
            .bearer_auth(&self.token.read().await)
            .json(&req);

        let response = request.send().await?;
        let text = response.text().await?;

        match serde_json::from_str(&text) {
            std::result::Result::Ok(json) => Ok(json),
            Err(e) => Err(anyhow::Error::new(e).context(text.trim().to_string())),
        }
    }

    pub async fn check_upload(
        &self,
        req: types::request::CheckUploadRequest,
    ) -> anyhow::Result<types::response::CheckUploadResponse> {
        if self.last_refresh.read().await.elapsed().as_secs() > self.token_valid_time.into() {
            self.refresh_token().await?;
        }
        let client = reqwest::Client::new();
        let request = client
            .post("https://forest.sendy.jp/cloud/service/file/v1/check/upload")
            .bearer_auth(&self.token.read().await)
            .json(&req);

        let response = request.send().await?;
        let text = response.text().await?;

        match serde_json::from_str(&text) {
            std::result::Result::Ok(json) => Ok(json),
            Err(e) => Err(anyhow::Error::new(e).context(text.trim().to_string())),
        }
    }

    pub async fn get_upload_token(
        &self,
    ) -> anyhow::Result<types::response::GetFileLinkTokenResponse> {
        if self.last_refresh.read().await.elapsed().as_secs() > self.token_valid_time.into() {
            self.refresh_token().await?;
        }
        let client = reqwest::Client::new();
        let request = client
            .get(format!(
                "https://forest.sendy.jp/cloud/service/file/v1/filelink/token?host_id={}&path={}",
                self.host_id, "hello"
            ))
            .bearer_auth(&self.token.read().await);

        let response = request.send().await?;
        let text = response.text().await?;

        match serde_json::from_str(&text) {
            std::result::Result::Ok(json) => Ok(json),
            Err(e) => Err(anyhow::Error::new(e).context(text.trim().to_string())),
        }
    }

    pub async fn get_download_link(
        &self,
        req: types::request::GetFileLinkRequest,
    ) -> anyhow::Result<types::response::GetFileLinkResponse> {
        if self.last_refresh.read().await.elapsed().as_secs() > self.token_valid_time.into() {
            self.refresh_token().await?;
        }
        let client = reqwest::Client::new();
        let request = client
            .post("https://forest.sendy.jp/cloud/service/file/v1/filelink/download")
            .bearer_auth(&self.token.read().await)
            .json(&req);

        let response = request.send().await?;
        let text = response.text().await?;

        match serde_json::from_str(&text) {
            std::result::Result::Ok(json) => Ok(json),
            Err(e) => Err(anyhow::Error::new(e).context(text.trim().to_string())),
        }
    }

    pub async fn check_action(
        &self,
        req: types::request::CheckActionRequest,
    ) -> anyhow::Result<types::response::CheckActionResponse> {
        if self.last_refresh.read().await.elapsed().as_secs() > self.token_valid_time.into() {
            self.refresh_token().await?;
        }
        let client = reqwest::Client::new();
        let request = client
            .post("https://forest.sendy.jp/cloud/service/file/v3/files/check")
            .bearer_auth(&self.token.read().await)
            .json(&req);

        let response = request.send().await?;
        let text = response.text().await?;

        match serde_json::from_str(&text) {
            std::result::Result::Ok(json) => Ok(json),
            Err(e) => Err(anyhow::Error::new(e).context(text.trim().to_string())),
        }
    }

    pub async fn file_detail(
        &self,
        req: types::request::FileDetailRequest,
    ) -> anyhow::Result<types::response::FileDetailResponse> {
        if self.last_refresh.read().await.elapsed().as_secs() > self.token_valid_time.into() {
            self.refresh_token().await?;
        }
        let client = reqwest::Client::new();
        let request = client
            .post("https://forest.sendy.jp/cloud/service/file/v1/file")
            .bearer_auth(&self.token.read().await)
            .json(&req);

        let response = request.send().await?;
        let text = response.text().await?;

        match serde_json::from_str(&text) {
            std::result::Result::Ok(json) => Ok(json),
            Err(err) => match serde_json::from_str::<types::response::SendyError>(&text) {
                std::result::Result::Ok(json) => Err(anyhow::anyhow!("{:?}", json)),
                Err(_) => Err(anyhow::Error::new(err).context(text.trim().to_string())),
            },
        }
    }

    pub async fn delete_file(
        &self,
        req: types::request::DeleteFileRequest,
    ) -> anyhow::Result<types::response::JobKeyResponse> {
        if self.last_refresh.read().await.elapsed().as_secs() > self.token_valid_time.into() {
            self.refresh_token().await?;
        }
        let client = reqwest::Client::new();
        let request = client
            .delete("https://forest.sendy.jp/cloud/service/file/v3/files")
            .bearer_auth(&self.token.read().await)
            .json(&req);

        let response = request.send().await?;
        let text = response.text().await?;

        match serde_json::from_str(&text) {
            std::result::Result::Ok(json) => Ok(json),
            Err(e) => Err(anyhow::Error::new(e).context(text.trim().to_string())),
        }
    }

    pub async fn mkdir(&self, req: types::request::CreateFolderRequest) -> anyhow::Result<()> {
        if self.last_refresh.read().await.elapsed().as_secs() > self.token_valid_time.into() {
            self.refresh_token().await?;
        }
        let client = reqwest::Client::new();
        let request = client
            .post("https://forest.sendy.jp/cloud/service/file/v1/files/create")
            .bearer_auth(&self.token.read().await)
            .json(&req);

        let response = request.send().await?;

        if response.status() == StatusCode::NO_CONTENT {
            Ok(())
        } else {
            let text = response.text().await?;
            Err(anyhow::anyhow!("Failed to create folder: {}", text))
        }
    }

    pub async fn copy_file(
        &self,
        req: types::request::CopyFileRequest,
    ) -> anyhow::Result<types::response::JobKeyResponse> {
        if self.last_refresh.read().await.elapsed().as_secs() > self.token_valid_time.into() {
            self.refresh_token().await?;
        }
        let client = reqwest::Client::new();
        let request = client
            .post("https://forest.sendy.jp/cloud/service/file/v3/files/copy")
            .bearer_auth(&self.token.read().await)
            .json(&req);

        let response = request.send().await?;
        let text = response.text().await?;

        match serde_json::from_str(&text) {
            std::result::Result::Ok(json) => Ok(json),
            Err(e) => Err(anyhow::Error::new(e).context(text.trim().to_string())),
        }
    }

    pub async fn rename_file(
        &self,
        req: types::request::RenameFileRequest,
    ) -> anyhow::Result<types::response::JobKeyResponse> {
        if self.last_refresh.read().await.elapsed().as_secs() > self.token_valid_time.into() {
            self.refresh_token().await?;
        }
        let client = reqwest::Client::new();
        let request = client
            .put("https://forest.sendy.jp/cloud/service/file/v3/files/rename")
            .bearer_auth(&self.token.read().await)
            .json(&req);

        let response = request.send().await?;
        let text = response.text().await?;

        match serde_json::from_str(&text) {
            std::result::Result::Ok(json) => Ok(json),
            Err(e) => Err(anyhow::Error::new(e).context(text.trim().to_string())),
        }
    }

    pub async fn move_file(
        &self,
        req: types::request::MoveFileRequest,
    ) -> anyhow::Result<types::response::JobKeyResponse> {
        if self.last_refresh.read().await.elapsed().as_secs() > self.token_valid_time.into() {
            self.refresh_token().await?;
        }
        let client = reqwest::Client::new();
        let request = client
            .put("https://forest.sendy.jp/cloud/service/file/v3/files/move")
            .bearer_auth(&self.token.read().await)
            .json(&req);

        let response = request.send().await?;
        let text = response.text().await?;

        match serde_json::from_str(&text) {
            std::result::Result::Ok(json) => Ok(json),
            Err(e) => Err(anyhow::Error::new(e).context(text.trim().to_string())),
        }
    }
}

pub async fn refresh_token(
    req: types::request::RefreshTokenRequest,
) -> anyhow::Result<types::response::RefreshTokenResponse> {
    let client = reqwest::Client::new();
    let request = client
        .post("https://www.rakuten-drive.com/api/account/refreshtoken")
        .json(&req);

    let response = request.send().await?;
    let text = response.text().await?;

    //

    let json: types::response::RefreshTokenResponse = serde_json::from_str(&text)?;
    Ok(json)

    // response
    //     .json::<types::response::RefreshTokenResponse>()
    //     .await
    //     .map_err(Into::into)
}

pub async fn rid_token_auth(rid_code: &str) -> anyhow::Result<types::response::RIDTokenResponse> {
    let client = reqwest::Client::new();
    let request = client
        .get(format!("https://www.rakuten-drive.com/api/v1/auth/rd/custom/token?rid_code={}&is_extension=false", rid_code))
        .send()
        .await?;

    let text = request.text().await?;

    let json: types::response::RIDTokenResponse = serde_json::from_str(&text)?;
    Ok(json)
}

pub async fn get_refresh_token(
    token: &str,
) -> anyhow::Result<types::response::VerifyCustomTokenResponse> {
    let client = reqwest::Client::new();
    let req = types::request::VerifyCustomTokenRequest {
        token: token.to_string(),
        return_secure_token: true,
    };
    let request = client
        .post("https://www.googleapis.com/identitytoolkit/v3/relyingparty/verifyCustomToken?key=AIzaSyDyp5IGr4nXbYin_oduNGi6ci-AnWcuAYE")
        .json(&req)
        .send()
        .await?;

    let text = request.text().await?;

    let json: types::response::VerifyCustomTokenResponse = serde_json::from_str(&text)?;
    Ok(json)
}

// https://www.rakuten-drive.com/api/account/refreshtoken POST RefreshTokenRequest RefreshTokenResponse
// https://forest.sendy.jp/cloud/service/file/v1/file POST FileDetailRequest FileDetailResponse
// https://forest.sendy.jp/cloud/service/file/v1/files POST ListFilesRequest ListFilesResponse
// https://forest.sendy.jp/cloud/service/file/v3/files DELETE DeleteFileRequest JobKeyResponse
// https://forest.sendy.jp/cloud/service/file/v1/files/create POST CreateFolderRequest
// https://forest.sendy.jp/cloud/service/file/v3/files/rename PUT RenameFileRequest RenameFileResponse
// https://forest.sendy.jp/cloud/service/file/v3/files/check POST CheckActionRequest CheckActionResponse
// https://forest.sendy.jp/cloud/service/file/v3/files/move PUT MoveFileRequest MoveFileResponse
// https://forest.sendy.jp/cloud/service/file/v1/check/upload POST CheckUploadRequest CheckUploadResponse
// https://forest.sendy.jp/cloud/service/file/v1/filelink/token?host_id=GclT7DrnLFho7vnIirUzjtMLhRk2&path=hello GET GetFileLinkTokenResponse
// https://forest.sendy.jp/cloud/service/file/v1/complete/upload POST CompleteUploadRequest
// https://forest.sendy.jp/cloud/service/file/v1/filelink/download POST GetFileLinkRequest GetFileLinkResponse
// https://forest.sendy.jp/cloud/service/file/v3/files/copy POST CopyFileRequest JobKeyResponse
