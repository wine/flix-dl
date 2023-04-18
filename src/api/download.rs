use std::{path::PathBuf, time::Duration};

use anyhow::Result;
use async_trait::async_trait;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

use super::{Client, Error};

#[async_trait]
pub trait Download {
    async fn download(&self, client: &Client, base_path: &PathBuf) -> Result<()>;
}

pub(crate) async fn download_with_progress_bar(
    client: &Client,
    link: &str,
    path: &PathBuf,
) -> Result<()> {
    let response = client.get(link)?.send().await?;

    let file_name = response
        .url()
        .query_pairs()
        .find(|(k, _)| k == "fn")
        .ok_or(Error::InvalidDownload)?
        .1
        .to_string();

    let path = path.join(&file_name);

    let total_size = response.content_length().ok_or(Error::InvalidDownload)?;
    if total_size == get_initial_position(&path).await {
        return Ok(());
    }

    let progress_bar = make_progress_bar(file_name, total_size)?;
    progress_bar.enable_steady_tick(Duration::from_millis(50));

    let mut stream = response.bytes_stream();
    let mut file = File::create(path).await?;

    while let Some(bytes) = stream.next().await {
        let bytes = bytes?;
        file.write_all(&bytes).await?;
        progress_bar.inc(bytes.len() as u64);
    }

    Ok(())
}

async fn get_initial_position(path: &PathBuf) -> u64 {
    match fs::metadata(path).await {
        Ok(metadata) => metadata.len(),
        Err(_) => 0,
    }
}

fn make_progress_bar(file_name: String, total_size: u64) -> Result<ProgressBar> {
    let progress_bar = ProgressBar::new(total_size);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{elapsed}] [{wide_bar:.green/white}] {bytes}/{total_bytes} {eta}")?
            .progress_chars("#>-"),
    );

    progress_bar.set_message(file_name);

    Ok(progress_bar)
}
