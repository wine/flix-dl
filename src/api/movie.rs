use std::path::PathBuf;

use anyhow::Result;
use async_trait::async_trait;
use select::{document::Document, predicate::Class};
use tokio::fs;

use super::{download_with_progress_bar, Client, Download};

#[derive(Debug)]
pub struct Movie {
    name: String,
    link: String,
}

impl TryFrom<Document> for Movie {
    type Error = anyhow::Error;

    fn try_from(doc: Document) -> Result<Self> {
        // FIXME: Don't unwrap
        let name = doc.find(Class("watch-header")).next().unwrap().text();

        // FIXME: Don't unwrap
        let id = doc
            .find(Class("favorite"))
            .next()
            .unwrap()
            .attr("data-pid")
            .unwrap();

        let link = format!("/download?id={id}");

        Ok(Movie {
            name: String::from(name),
            link,
        })
    }
}

#[async_trait]
impl Download for Movie {
    async fn download(&self, client: &Client, base_path: &PathBuf) -> Result<()> {
        let path = base_path.join(&self.name);
        fs::create_dir_all(&path).await?;

        let path = path.join(&format!("{}.mp4", self.name));
        download_with_progress_bar(&client, &self.link, &path).await
    }
}
