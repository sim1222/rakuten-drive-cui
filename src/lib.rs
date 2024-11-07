use aws_config::Region;
use aws_sdk_s3::config::Credentials;
use constants::APP_VERSION;
use std::{io::BufReader, path::PathBuf, sync::Arc};
use util::{check_job, file_detail, list_files, multipart_upload, TargetFile};

mod client;
mod constants;
mod types;
mod util;

pub struct RakutenDriveClient {
    client: client::Client,
}

impl RakutenDriveClient {
    pub async fn try_new(refresh_token_str: String) -> anyhow::Result<Self> {
        let client = client::Client::try_new(refresh_token_str).await?;
        Ok(Self { client })
    }
    pub async fn list(
        &self,
        prefix: Option<&str>,
    ) -> anyhow::Result<types::response::ListFilesResponse> {
        list_files(Some(prefix.unwrap_or("")), &self.client).await
    }
    pub async fn info(&self, path: &str) -> anyhow::Result<types::response::FileDetailResponse> {
        let req = types::request::FileDetailRequest {
            host_id: self.client.host_id.clone(),
            path: path.to_string(),
            thumbnail_size: 130,
        };
        self.client.file_detail(req).await
    }
    pub async fn upload(
        &self,
        file_path: &str,
        file_data: &[u8],
        prefix: Option<&str>,
        fake_size: Option<u64>,
        pb: Option<indicatif::ProgressBar>,
    ) -> anyhow::Result<()> {
        let req = types::request::CheckUploadRequest {
            host_id: self.client.host_id.clone(),
            path: prefix.unwrap_or("").to_string(),
            upload_id: "".to_string(),
            file: vec![types::request::CheckUploadRequestFile {
                path: file_path.to_string(),
                size: fake_size.unwrap_or(file_data.len() as u64) as i64,
            }],
        };

        let check_upload_res = self.client.check_upload(req).await.unwrap();

        let token_res = self.client.get_upload_token().await.unwrap();

        let cledential = Credentials::new(
            token_res.access_key_id.clone(),
            token_res.secret_access_key.clone(),
            Some(token_res.session_token.clone()),
            None,
            "2021-06-01",
        );
        let _config = aws_sdk_s3::Config::builder()
            .behavior_version_latest()
            .region(Region::new(check_upload_res.region.clone()))
            .credentials_provider(cledential)
            .force_path_style(true)
            .build();

        multipart_upload(
            &token_res,
            &check_upload_res.bucket,
            &check_upload_res.file[0],
            &check_upload_res.prefix,
            &check_upload_res.region,
            &check_upload_res.upload_id,
            file_data,
            pb,
        )
        .await
        .unwrap();
        // if file_size > CHUNK_SIZE as u64 {
        // for (i, file) in files.iter().enumerate() {
        //     println!("Multi Uploading: {:?}", file.file);

            //     }
            // } else {
            //     for (i, file) in files.iter().enumerate() {
            //         println!("Uploading: {:?}", file.file);
            //         let stream = ByteStream::read_from()
            //             .path(file.file.clone())
            //             .offset(0)
            //             .length(Length::Exact(file_size))
            //             .build()
            //             .await
            //             .unwrap();
            //         let key =
            //             check_upload_res.prefix.to_owned() + check_upload_res.file[i].path.as_str();
            //         let _upload_res = s3_client
            //             .put_object()
            //             .bucket(check_upload_res.bucket.clone())
            //             .key(key)
            //             .body(stream)
            //             .send()
            //             .await
            //             .unwrap();
            //     }
            // }
        // }

        match check_job(&check_upload_res.upload_id, &self.client).await {
            Ok(_) => Ok(()),
            Err(e) => {
                println!("Error: {:?}", e);
                Err(anyhow::anyhow!("Error: {:?}", e))
            }
        }
    }

    pub async fn download(&self, path: &str, prefix: Option<&str>) -> anyhow::Result<()> {
        let _file_name = path.split('/').last().unwrap();
        let file_path =
            path.split('/').collect::<Vec<&str>>()[0..path.split('/').count() - 1].join("/");

        let file = match file_detail(path, &self.client).await {
            Ok(file) => file,
            Err(e) => {
                return Err(e);
            }
        };

        let req = types::request::GetFileLinkRequest {
            app_version: APP_VERSION.to_string(),
            file: vec![types::request::GetFileLinkRequestFile {
                path: path.to_string(),
                size: file.size,
            }],
            host_id: self.client.host_id.clone(),
            path: file_path,
        };

        let res = self.client.get_download_link(req).await.unwrap();

        // run aria2c

        // TODO: Implement self implementation of multi connection download
        let stdout = std::process::Command::new("aria2c")
            .arg("-x16")
            .arg("-s16")
            .arg("-d")
            .arg(".")
            .arg(res.url)
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("failed to execute process")
            .stdout
            .expect("failed to get stdout");

        let reader = std::io::BufReader::new(stdout);

        std::io::BufRead::lines(reader).for_each(|line| println!("{}", line.unwrap()));

        Ok(())
    }

    pub async fn mkdir(&self, name: &str, path: Option<&str>) -> anyhow::Result<()> {
        if name.contains('/') {
            println!("Please use --path option for set parent directory");
            return Err(anyhow::anyhow!(
                "Please use --path option for set parent directory"
            ));
        }
        let req = types::request::CreateFolderRequest {
            host_id: self.client.host_id.clone(),
            name: name.to_string(),
            path: path.unwrap_or("").to_string(),
        };

        match self.client.mkdir(req).await {
            Ok(_) => {
                println!("Created: {:?}", name);
                Ok(())
            }
            Err(e) => {
                println!("Error: {:?}", e);
                Err(anyhow::anyhow!("Error: {:?}", e))
            }
        }
    }

    pub async fn rename(&self, path: &str, name: &str) -> anyhow::Result<()> {
        if name.contains('/') {
            println!("Can't use / in file name");
            println!("Name should be file name only.");
            return Err(anyhow::anyhow!("Can't use / in file name"));
        }

        let file_path =
            path.split('/').collect::<Vec<&str>>()[0..path.split('/').count() - 1].join("/") + "/";

        let file = match file_detail(path, &self.client).await {
            Ok(file) => file,
            Err(e) => {
                return Err(e);
            }
        };

        let req = types::request::RenameFileRequest {
            file: types::request::FileModifyRequestFile {
                last_modified: file.last_modified.clone(),
                path: file.path.clone(),
                version_id: file.version_id.clone(),
                size: file.size,
            },
            host_id: self.client.host_id.clone(),
            name: name.to_string(),
            prefix: file_path,
        };

        let res = self.client.rename_file(req).await.unwrap();

        match check_job(&res.key, &self.client).await {
            Ok(_) => {
                println!("Renamed.");
                Ok(())
            }
            Err(e) => {
                println!("Error: {:?}", e);
                Err(anyhow::anyhow!("Error: {:?}", e))
            }
        }
    }

    pub async fn move_file(&self, path: &str, dest: &str) -> anyhow::Result<()> {
        if !dest.ends_with('/') {
            println!("Destination should be directory.");
            return Err(anyhow::anyhow!("Destination should be directory."));
        }
        let file = file_detail(path, &self.client).await.unwrap();

        let file_name = path.split('/').last().unwrap();
        let file_dir =
            path.split('/').collect::<Vec<&str>>()[0..path.split('/').count() - 1].join("/") + "/";

        if (file_detail((dest.to_string() + file_name).as_str(), &self.client).await).is_ok() {
            println!("File already exists.");
            return Err(anyhow::anyhow!("File already exists."));
        }

        let req = types::request::MoveFileRequest {
            file: vec![types::request::FileModifyRequestFile {
                last_modified: file.last_modified,
                path: file.path,
                size: file.size,
                version_id: file.version_id,
            }],
            host_id: self.client.host_id.clone(),
            prefix: file_dir.clone(),
            target_id: self.client.host_id.clone(),
            to_path: dest.to_string(),
        };

        let res = self.client.move_file(req).await.unwrap();

        match check_job(&res.key, &self.client).await {
            Ok(_) => {
                println!("Moved.");
                Ok(())
            }
            Err(e) => {
                println!("Error: {:?}", e);
                Err(anyhow::anyhow!("Error: {:?}", e))
            }
        }
    }

    pub async fn delete(&self, path: &str, recursive: &bool) -> anyhow::Result<()> {
        let file = file_detail(path, &self.client).await.unwrap();
        if file.is_folder && !*recursive {
            println!("Use --recursive option for folder delete");
            return Err(anyhow::anyhow!("Use --recursive option for folder delete"));
        }
        let req = types::request::DeleteFileRequest {
            file: vec![types::request::FileModifyRequestFile {
                last_modified: file.last_modified,
                path: file.path,
                version_id: file.version_id,
                size: file.size,
            }],
            host_id: self.client.host_id.clone(),
            prefix: "".to_string(),
            trash: true,
        };
        let res = self.client.delete_file(req).await.unwrap();

        match check_job(&res.key, &self.client).await {
            Ok(_) => {
                println!("Deleted.");
                Ok(())
            }
            Err(e) => {
                println!("Error: {:?}", e);
                Err(anyhow::anyhow!("Error: {:?}", e))
            }
        }
    }

    pub async fn copy(&self, src: &str, dest: &str) -> anyhow::Result<()> {
        if !dest.ends_with('/') {
            println!("Destination should be directory.");
            return Err(anyhow::anyhow!("Destination should be directory."));
        }
        let file_name = src.split('/').last().unwrap();
        let file_dir =
            src.split('/').collect::<Vec<&str>>()[0..src.split('/').count() - 1].join("/") + "/";

        let file = file_detail(src, &self.client).await.unwrap();

        if (file_detail((dest.to_string() + file_name).as_str(), &self.client).await).is_ok() {
            println!("File already exists.");
            return Err(anyhow::anyhow!("File already exists."));
        }

        let req = types::request::CopyFileRequest {
            file: vec![types::request::CopyFileRequestFile {
                last_modified: file.last_modified,
                path: file.path,
                version_id: file.version_id,
                size: file.size,
            }],
            host_id: self.client.host_id.clone(),
            prefix: file_dir.clone(),
            target_id: self.client.host_id.clone(),
            to_path: dest.to_string(),
        };

        let res = self.client.copy_file(req).await.unwrap();

        match check_job(&res.key, &self.client).await {
            Ok(_) => {
                println!("Copied.");
                Ok(())
            }
            Err(e) => {
                println!("Error: {:?}", e);
                Err(anyhow::anyhow!("Error: {:?}", e))
            }
        }
    }
}
