use std::path::PathBuf;

use anyhow::Result;
use async_trait::async_trait;
use select::{
    document::Document,
    predicate::{Attr, Class},
};
use tokio::fs;

use super::{download_with_progress_bar, Client, Download};

#[derive(Debug)]
pub struct Show {
    name: String,
    seasons: Vec<Season>,
}

#[derive(Debug)]
pub struct Season {
    number: u32,
    episodes: Vec<Episode>,
}

#[derive(Clone, Debug)]
pub struct Episode {
    season: u32,
    number: u32,
    name: String,
    link: String,
}

impl TryFrom<Document> for Show {
    type Error = anyhow::Error;

    fn try_from(doc: Document) -> Result<Self, Self::Error> {
        // FIXME: Don't unwrap
        let show_name = doc.find(Class("watch-header")).next().unwrap().text();

        let mut show = Show {
            name: String::from(show_name),
            seasons: Vec::new(),
        };

        for season_node in doc.find(Class("section-watch-season")) {
            // FIXME: Don't unwrap
            let season_number = season_node
                .find(Attr("itemprop", "seasonNumber"))
                .next()
                .unwrap()
                .text()
                .parse::<u32>()?;

            let mut season = Season {
                number: season_number,
                episodes: Vec::new(),
            };

            for episode in season_node.find(Class("eplist")) {
                // FIXME: Don't unwrap
                let episode_number = episode
                    .find(Attr("itemprop", "episodeNumber"))
                    .next()
                    .unwrap()
                    .text()
                    .parse::<u32>()?;

                // FIXME: Don't unwrap
                let episode_name = episode
                    .find(Attr("itemprop", "name"))
                    .next()
                    .unwrap()
                    .text();

                // FIXME: Don't unwrap
                let episode_link = episode
                    .find(Class("downloadvid"))
                    .next()
                    .unwrap()
                    .attr("data-href")
                    .unwrap();

                season.episodes.push(Episode {
                    season: season_number,
                    number: episode_number,
                    name: episode_name,
                    link: String::from(episode_link),
                });
            }

            season.episodes.reverse();
            show.seasons.push(season);
        }

        show.seasons.reverse();

        Ok(show)
    }
}

#[async_trait]
impl Download for Show {
    async fn download(&self, client: &Client, base_path: &PathBuf) -> Result<()> {
        let path = base_path.join(&self.name);
        for season in &self.seasons {
            season.download(&client, &path).await?;
        }

        Ok(())
    }
}

#[async_trait]
impl Download for Season {
    async fn download(&self, client: &Client, base_path: &PathBuf) -> Result<()> {
        let path = base_path.join(format!("Season {}", self.number));
        fs::create_dir_all(&path).await?;
        for episode in &self.episodes {
            episode.download(&client, &path).await?;
        }

        Ok(())
    }
}

#[async_trait]
impl Download for Episode {
    async fn download(&self, client: &Client, base_path: &PathBuf) -> Result<()> {
        let path = base_path.join(format!(
            "S{:02}.E{:02}_{}.mp4",
            self.season, self.number, self.name
        ));
        download_with_progress_bar(&client, &self.link, &path).await
    }
}