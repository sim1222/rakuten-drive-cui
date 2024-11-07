use std::{
    cmp::{max, min},
    io::{stdout, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use aws_config::{environment::region, BehaviorVersion, Region, SdkConfig};
use aws_sdk_s3::{
    config::Credentials, operation::upload_part, primitives::ByteStream, types::CompletedPart,
};
use aws_smithy_runtime::client::http::hyper_014::HyperClientBuilder;
use aws_smithy_types::byte_stream::Length;
use clap::{Parser, Subcommand};
use human_bytes::human_bytes;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use tokio::{fs::File, io::BufReader, sync::Mutex};

use crate::{
    client::{self},
    types::response::ListFilesResponseFile,
};
use crate::{constants::CHUNK_SIZE, types};

#[derive(Debug, Clone)]
pub struct TargetFile {
    pub file: PathBuf,
    pub path: String,
}

pub async fn multipart_upload(
    token_res: &types::response::GetFileLinkTokenResponse,
    bucket: &str,
    target_file: &types::response::CheckUploadResponseFile,
    prefix: &str,
    region: &str,
    upload_id: &str,
    file: &[u8],
    pb: Option<ProgressBar>,
) -> anyhow::Result<()> {
    let _ = upload_id;
    // if !file.file.exists() {
    //     println!("File not found: {:?}", file.file);
    //     return Err(anyhow::anyhow!("File not found: {:?}", file.file));
    // }

    let file_size = file.len() as u64;

    let cledential = Credentials::new(
        &token_res.access_key_id,
        &token_res.secret_access_key,
        Some(token_res.session_token.clone()),
        // 2024-07-18T07:14:42Z
        Some(
            chrono::DateTime::parse_from_rfc3339(&token_res.expiration)
                .unwrap()
                .into(),
        ),
        "2021-06-01",
    );

    let config = aws_sdk_s3::Config::builder()
        .behavior_version_latest()
        .credentials_provider(cledential)
        .region(Region::new(region.to_owned()))
        // .endpoint_url("https://sendy-cloud.s3.ap-northeast-1.amazonaws.com")
        .build();

    let s3_client = aws_sdk_s3::Client::from_conf(config);

    let key = prefix.to_owned() + target_file.path.as_str();

    let multipart_upload_res = s3_client
        .create_multipart_upload()
        .bucket(bucket)
        .key(key.clone())
        .send()
        .await
        .unwrap();

    let upload_id = multipart_upload_res.upload_id().unwrap().to_string();

    let chunk_size = max(CHUNK_SIZE as u64, file_size / 10000);

    let mut chunk_count = file_size / chunk_size;
    let mut size_of_last_chunk = file_size % chunk_size;

    if size_of_last_chunk == 0 {
        size_of_last_chunk = chunk_size;
        chunk_count -= 1;
    }

    let upload_parts = Arc::new(Mutex::new(Vec::<CompletedPart>::new()));

    let semaphore = Arc::new(tokio::sync::Semaphore::new(20));
    let mut handles = Vec::new();

    for chunk_index in 0..chunk_count {
        let bucket = bucket.to_owned();
        let key = key.clone();
        let upload_id = upload_id.clone();
        let s3_client = s3_client.clone();
        let pb = pb.clone();
        let file = file.to_owned();
        let upload_parts = upload_parts.clone();

        let semaphore = semaphore.clone().acquire_owned().await.unwrap();

        let handle = tokio::spawn(async move {
            let _permit = semaphore;

            let this_chunk = if chunk_count - 1 == chunk_index {
                size_of_last_chunk
            } else {
                chunk_size
            };
            loop {
                let offset = chunk_index * chunk_size;
                let length = this_chunk;

                let bytes = file[offset as usize..(offset + length) as usize].to_vec();
                let stream = ByteStream::from(bytes);
                // let stream = match stream {
                //     Ok(stream) => stream,
                //     Err(e) => {
                //         eprintln!("Error: {:?}", e);
                //         continue;
                //     }
                // };
                //Chunk index needs to start at 0, but part numbers start at 1.
                let part_number = (chunk_index as i32) + 1;
                let upload_part_res = s3_client
                    .upload_part()
                    .key(&key)
                    .bucket(bucket.clone())
                    .upload_id(upload_id.clone())
                    .body(stream)
                    .part_number(part_number)
                    .send()
                    .await;
                let upload_part_res = match upload_part_res {
                    Ok(upload_part_res) => upload_part_res,
                    Err(e) => {
                        eprintln!("Error: {:?}", e);
                        continue;
                    }
                };
                upload_parts.lock().await.push(
                    CompletedPart::builder()
                        .e_tag(upload_part_res.e_tag.unwrap_or_default())
                        .part_number(part_number)
                        .build(),
                );
                if let Some(pb) = &pb {
                    pb.inc(this_chunk);
                }
                break;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    upload_parts
        .lock()
        .await
        .sort_by(|a, b| a.part_number.cmp(&b.part_number));

    let completed_multipart_upload = aws_sdk_s3::types::CompletedMultipartUpload::builder()
        .set_parts(Some(upload_parts.lock().await.clone()))
        .build();

    let _complete_multipart_upload_res = s3_client
        .complete_multipart_upload()
        .bucket(bucket)
        .key(key)
        .upload_id(upload_id)
        .multipart_upload(completed_multipart_upload)
        .send()
        .await
        .unwrap();

    if let Some(pb) = pb {
        pb.finish_with_message("Uploaded");
    }

    Ok(())
}

pub async fn file_detail(
    path: &str,
    client: &client::Client,
) -> anyhow::Result<types::response::FileDetailResponseFile> {
    let req = types::request::FileDetailRequest {
        host_id: client.host_id.clone(),
        path: path.to_string(),
        thumbnail_size: 130,
    };
    let res = client.file_detail(req).await?;
    Ok(res.file)
}

pub async fn list_files(
    prefix: Option<&str>,
    client: &client::Client,
) -> anyhow::Result<types::response::ListFilesResponse> {
    let pagination_size = 40;
    let mut files = Vec::<ListFilesResponseFile>::new();
    let req = types::request::ListFilesRequest {
        from: 0,
        host_id: client.host_id.clone(),
        path: prefix.unwrap_or("").to_string(),
        sort_type: types::request::ListFilesRequestSortType::Path,
        reverse: false,
        thumbnail_size: 130,
        to: pagination_size,
    };
    let mut res = client.list_files(req).await?;

    files.append(&mut res.file);

    if !res.last_page {
        let mut cursor = res.file.len() as i64;
        loop {
            let req = types::request::ListFilesRequest {
                from: cursor,
                host_id: client.host_id.clone(),
                path: prefix.unwrap_or("").to_string(),
                sort_type: types::request::ListFilesRequestSortType::Path,
                reverse: false,
                thumbnail_size: 130,
                to: pagination_size + cursor,
            };

            let mut next_res = client.list_files(req).await?;
            files.append(&mut next_res.file);

            if next_res.last_page {
                break;
            } else {
                cursor += next_res.file.len() as i64;
            }
        }
    }
    res.file = files;

    // files.iter().find(|f| f.path == "/").unwrap();

    Ok(res)
}

pub async fn check_job(key: &str, client: &client::Client) -> anyhow::Result<()> {
    loop {
        let req = types::request::CheckActionRequest {
            key: key.to_string(),
        };
        let res = client.check_action(req).await.unwrap();

        if res.state == "complete" {
            return Ok(());
        }
        if res.state == "error" {
            println!("Error: {:?}", res);
            return Err(anyhow::anyhow!("Error: {:?}", res));
        }

        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}
