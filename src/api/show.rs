use std::{path::PathBuf, time::Duration};

use anyhow::Result;
use async_trait::async_trait;
use select::{
    document::Document,
    node::Data,
    predicate::{Attr, Class},
};
use tokio::{fs, time};

use super::{download_with_progress_bar, Client, Download, Error};

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
    _name: String,
    link: String,
}

impl TryFrom<Document> for Show {
    type Error = anyhow::Error;

    fn try_from(doc: Document) -> Result<Self, Self::Error> {
        let show_name = doc
            .find(Class("watch-header"))
            .next()
            .ok_or(Error::MissingClass("watch-header"))?
            .children()
            .filter(|node| match node.data() {
                Data::Text(_) => true,
                _ => false,
            })
            .map(|node| node.text())
            .collect::<String>();

        let show_name = show_name.trim();

        let mut show = Show {
            name: String::from(show_name),
            seasons: Vec::new(),
        };

        for season_node in doc.find(Class("section-watch-season")) {
            let season_number = season_node
                .find(Attr("itemprop", "seasonNumber"))
                .next()
                .ok_or(Error::MissingAttr("itemprop::seasonNumber"))?
                .text()
                .parse::<u32>()?;

            let mut season = Season {
                number: season_number,
                episodes: Vec::new(),
            };

            for episode in season_node.find(Class("eplist")) {
                let episode_number = episode
                    .find(Attr("itemprop", "episodeNumber"))
                    .next()
                    .ok_or(Error::MissingAttr("itemprop::episodeNumber"))?
                    .text()
                    .parse::<u32>()?;

                let episode_name = episode
                    .find(Attr("itemprop", "name"))
                    .next()
                    .ok_or(Error::MissingAttr("itemprop::name"))?
                    .text();

                let episode_link = episode
                    .find(Class("downloadvid"))
                    .next()
                    .ok_or(Error::MissingClass("downloadvid"))?
                    .attr("data-href")
                    .ok_or(Error::MissingAttr("data-href"))?;

                season.episodes.push(Episode {
                    season: season_number,
                    number: episode_number,
                    _name: episode_name,
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
    async fn prefetch(&self, client: &Client) -> Result<()> {
        for season in &self.seasons {
            season.prefetch(&client).await?;
        }

        Ok(())
    }

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
    async fn prefetch(&self, client: &Client) -> Result<()> {
        for episode in &self.episodes {
            episode.prefetch(&client).await?;
        }

        Ok(())
    }

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
    async fn prefetch(&self, client: &Client) -> Result<()> {
        println!("Prefetching S{:02}E{:02}", self.season, self.number);

        let response = client.get(&self.link)?.send().await?;
        if let None = response.content_length() {
            // approx. 20 conversion requests per 300 seconds
            // 20 / 300  = 0.067
            // 1 / 0.067 = 14.925 seconds
            // FIXME: better rate limiting
            time::sleep(Duration::from_secs(18)).await;
        }

        Ok(())
    }

    async fn download(&self, client: &Client, base_path: &PathBuf) -> Result<()> {
        download_with_progress_bar(&client, &self.link, &base_path).await
    }
}
