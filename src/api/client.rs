use std::sync::Arc;

use anyhow::Result;
use reqwest::{cookie::Jar, Method, RequestBuilder, Url};
use select::document::Document;

use super::{Movie, Show};

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; rv:112.0) Gecko/20100101 Firefox/112.0";

#[derive(Clone)]
pub struct Client {
    base_url: Url,
    client: reqwest::Client,
}

impl Client {
    pub fn new(base_url: &str, vip_token: &str) -> Result<Self> {
        let base_url = Url::parse(base_url)?;

        let jar = Jar::default();
        jar.add_cookie_str(&format!("vipLogin={vip_token}"), &base_url);

        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .cookie_provider(Arc::new(jar))
            .build()?;

        Ok(Self { base_url, client })
    }

    pub async fn get_movie(&self, id: u32) -> Result<Movie> {
        let doc = self.get_document(&format!("/watch/movie/{id}")).await?;
        Movie::try_from(doc)
    }

    pub async fn get_show(&self, id: u32) -> Result<Show> {
        let doc = self.get_document(&format!("/watch/tv/{id}")).await?;
        Show::try_from(doc)
    }

    fn request(&self, method: Method, url: &str) -> Result<RequestBuilder> {
        let url = self.make_url(url)?;
        Ok(self.client.request(method, url))
    }

    pub fn get(&self, url: &str) -> Result<RequestBuilder> {
        self.request(Method::GET, url)
    }

    async fn get_document(&self, url: &str) -> Result<Document> {
        let response = self.get(url)?.send().await?;
        let response = response.text().await?;
        Ok(Document::from(response.as_ref()))
    }

    fn make_url(&self, url: &str) -> Result<Url> {
        Ok(self.base_url.join(url)?)
    }
}
