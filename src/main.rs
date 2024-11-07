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
use constants::REFRESH_TOKEN;
use human_bytes::human_bytes;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use rakuten_drive_cui::RakutenDriveClient;
use tokio::{
    fs::File,
    io::{AsyncReadExt, BufReader},
    sync::Mutex,
};
use types::response::ListFilesResponseFile;
use util::*;

mod client;
mod constants;
mod types;
mod util;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[clap(about = "List files")]
    List {
        /// Parent folder path
        #[clap(short, long)]
        prefix: Option<String>,
    },
    #[clap(about = "Upload file")]
    Upload {
        file: PathBuf,

        /// Parent folder path
        #[clap(short, long)]
        prefix: Option<String>,

        /// Upload folder recursively
        #[clap(short, long)]
        recursive: bool,

        /// Send fake file size to server (byte)
        #[clap(short, long)]
        fake_size: Option<u64>,
    },
    #[clap(about = "Download file")]
    Download {
        path: String,

        /// Parent folder path
        #[clap(long)]
        prefix: Option<String>,
    },
    #[clap(about = "Move file")]
    Move {
        // Source file path
        path: String,

        // Destination folder path
        dest: String,
    },
    #[clap(about = "Delete file")]
    Delete {
        path: String,

        /// Delete folder recursively
        #[clap(long)]
        recursive: bool,
    },
    #[clap(about = "Make directory")]
    Mkdir {
        name: String,

        /// Path to create directory
        #[clap(long)]
        path: Option<String>,
    },
    #[clap(about = "Copy file")]
    Copy {
        /// Source file path
        src: String,

        /// Destination file directory
        dest: String,
    },
    #[clap(about = "Rename file")]
    Rename {
        /// Target file path
        path: String,

        /// New file name
        name: String,
    },
    #[clap(about = "Print file detail")]
    Info {
        path: String,
    },
    Auth {},
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let client = RakutenDriveClient::try_new(REFRESH_TOKEN.to_string()).await?;

    match &args.command {
        Commands::List { prefix } => {
            client.list(prefix.as_deref()).await.unwrap();
        }
        Commands::Upload {
            file,
            prefix,
            recursive,
            fake_size,
        } => {
            // is folder
            if file.is_dir() && !*recursive {
                println!("Use --recursive option for folder upload");
                return Err(anyhow::anyhow!("Use --recursive option for folder upload"));
            }

            let mut files = Vec::<TargetFile>::new();

            if file.is_dir() && *recursive {
                // upload folder
                let mut dirs = Vec::<PathBuf>::new();
                dirs.push(file.clone());
                while let Some(dir) = dirs.pop() {
                    let entries = std::fs::read_dir(dir).unwrap();
                    for entry in entries {
                        let entry = entry.unwrap();
                        let path = entry.path();
                        if path.is_dir() {
                            dirs.push(path);
                        } else {
                            files.push(TargetFile {
                                file: path.clone(),
                                path: path
                                    .strip_prefix(file)
                                    .unwrap()
                                    .to_str()
                                    .expect("Invalid File Name")
                                    .to_string(),
                            });
                        }
                    }
                }
                // for file in files {
                //     println!("{:?}", file);
                // }
            } else {
                // file check
                if !file.exists() {
                    println!("File not found: {:?}", file);
                    return Err(anyhow::anyhow!("File not found: {:?}", file));
                }
                files.push(TargetFile {
                    file: file.clone(),
                    path: file.file_name().unwrap().to_str().unwrap().to_string(),
                });
            }

            if cfg!(windows) {
                // replase \ to /
                files.iter_mut().for_each(|f| {
                    f.path = f.path.replace('\\', "/");
                });
            }

            for file in &files {
                if client.info(file.path.as_str()).await.is_ok() {
                    println!("File already exists.");
                    return Err(anyhow::anyhow!("File already exists."));
                }
            }

            for file in &files {
                let file_size = file.file.metadata().unwrap().len();
                let file_data = File::open(file.file.clone()).await?;
                let mut file_reader = tokio::io::BufReader::new(file_data);
                let mut file_data: Vec<u8> = Vec::with_capacity(file_size as usize);
                file_reader.read_to_end(&mut file_data).await?;

                let pb = ProgressBar::new(file_size);
                pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                    .unwrap()
                    .with_key("eta", |state: &ProgressState, w: &mut dyn std::fmt::Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
                    .progress_chars("#>-"));

                client
                    .upload(
                        &file.path,
                        &file_data,
                        prefix.as_deref(),
                        *fake_size,
                        Some(pb),
                    )
                    .await
                    .unwrap();
            }
        }
        Commands::Download { path, prefix } => {
            client
                .download(path.as_str(), prefix.as_deref())
                .await
                .unwrap();
        }
        Commands::Move { path, dest } => {
            client.move_file(path, dest).await.unwrap();
        }
        Commands::Delete { path, recursive } => {
            client.delete(path, recursive).await.unwrap();
        }
        Commands::Mkdir { name, path } => {
            client.mkdir(name, path.as_deref()).await.unwrap();
        }
        Commands::Copy { src, dest } => {
            client.copy(src, dest).await.unwrap();
        }
        Commands::Rename { path, name } => {
            client.rename(path, name).await.unwrap();
        }
        Commands::Info { path } => {
            client.info(path).await.unwrap();
        }
        Commands::Auth {} => {
            println!("Click the link below to authorize the app:\n");
            let link = "https://login.account.rakuten.com/sso/authorize?response_type=code&client_id=rakuten_drive_web&redirect_uri=https://www.rakuten-drive.com/oauth-callback&scope=openid+profile+email&prompt=login&ui_locales=en";
            println!("{}\n", link);

            println!("Paste the URL you were redirected to:");
            let mut auth_url = String::new();
            std::io::stdin().read_line(&mut auth_url).unwrap();
            let auth_url = url::Url::parse(auth_url.trim())?;

            let params = auth_url.query_pairs().collect::<Vec<_>>();

            let rid_code = params
                .iter()
                .find(|(key, _)| key == "code")
                .map(|(_, value)| value.to_string())
                .ok_or_else(|| anyhow::anyhow!("Code not found in URL"))?;

            let rid_token_auth_res = client::rid_token_auth(rid_code.as_str()).await?;
            let token_verify_res =
                client::get_refresh_token(&rid_token_auth_res.custom_token).await?;

            println!("Refresh token: {}", token_verify_res.refresh_token);
        }
    }

    Ok(())
}
