use std::path::PathBuf;

use anyhow::Result;
use async_trait::async_trait;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::{fs::File, io::AsyncWriteExt};

use super::Client;

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

    // FIXME: Don't unwrap
    let total_size = response.content_length().unwrap();
    let progress_bar = make_progress_bar(&path, total_size)?;

    let mut stream = response.bytes_stream();
    let mut file = File::create(path).await?;

    while let Some(bytes) = stream.next().await {
        let bytes = bytes?;
        file.write_all(&bytes).await?;
        progress_bar.inc(bytes.len() as u64);
    }

    Ok(())
}

fn make_progress_bar(path: &PathBuf, total_size: u64) -> Result<ProgressBar> {
    let progress_bar = ProgressBar::new(total_size);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{msg} {bar:40.green/black} {bytes:>11.green}/{total_bytes:<11.green} {bytes_per_sec:>13.red} eta {eta:.blue}")?
            .progress_chars("█▇▆▅▄▃▂▁  "),
    );

    // FIXME: Don't unwrap
    progress_bar.set_message(path.file_name().unwrap().to_str().unwrap().to_owned());
    
    Ok(progress_bar)
}
