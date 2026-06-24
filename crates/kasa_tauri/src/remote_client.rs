use std::env;

use anyhow::Result;
use kasa_core::{
    db::{
        TagQueryOutput,
        embeddings::EmbeddingDistance,
        schema::{Media, MediaSource},
    },
    media::{MediaInfo, SourceCategoryGroupedTags, TagWithDetails},
    tags::{AllTagsOrderingCriteria, TagWithCount, search::SearchCriteria},
};

#[derive(Default)]
pub struct RemoteClient {
    reqwest_client: reqwest::Client,
    base_url: String,
}

// my plan was to generate these using openapi, but theres no rust crate that can do it from
// openapi 3.1, so i am stuck with manually implementing these
impl RemoteClient {
    pub fn new(base_url: String) -> Self {
        // brotli should be enabled by default if i enable the feature but just in case
        let mut reqwest_client = reqwest::Client::builder().brotli(true);

        if env::var("KASA_REMOTE_CLIENT_VERBOSE") == Ok("1".to_string()) {
            reqwest_client = reqwest_client.connection_verbose(true);
        }

        Self {
            reqwest_client: reqwest_client.build().unwrap(),
            base_url,
        }
    }

    pub async fn ping(&self) -> Result<String> {
        let url = format!("{}/ping", self.base_url);
        let response = self.reqwest_client.get(&url).send().await?;
        Ok(response.text().await?)
    }

    pub async fn get_thumbnail(&self, hash: &str) -> Result<Vec<u8>> {
        let url = format!("{}/get_thumbnail?hash={}", self.base_url, hash);
        let response = self.reqwest_client.get(&url).send().await?;
        Ok(response.bytes().await?.to_vec())
    }

    pub async fn query_tags(&self, query: &str, limit: i64) -> Result<Vec<TagQueryOutput>> {
        let url = format!(
            "{}/query_tags?query={}&limit={}",
            self.base_url, query, limit
        );
        let response: Vec<TagQueryOutput> =
            self.reqwest_client.get(&url).send().await?.json().await?;

        Ok(response)
    }

    pub async fn get_info(&self, hash: &str) -> Result<Option<MediaInfo>> {
        let url = format!("{}/get_info?hash={}", self.base_url, hash);
        let response: Option<MediaInfo> =
            self.reqwest_client.get(&url).send().await?.json().await?;

        Ok(response)
    }

    pub async fn get_tags(&self, hash: &str) -> Result<Vec<TagWithDetails>> {
        let url = format!("{}/get_tags?hash={}", self.base_url, hash);
        let response: Vec<TagWithDetails> =
            self.reqwest_client.get(&url).send().await?.json().await?;

        Ok(response)
    }

    pub async fn get_media_type(&self, hash: &str) -> Result<String> {
        let url = format!("{}/get_media_type?hash={}", self.base_url, hash);
        let response: String = self.reqwest_client.get(&url).send().await?.json().await?;

        Ok(response)
    }

    pub async fn get_tags_grouped_by_source_categories(
        &self,
        hash: &str,
    ) -> Result<SourceCategoryGroupedTags> {
        let url = format!(
            "{}/get_tags_grouped_by_source_categories?hash={}",
            self.base_url, hash
        );
        let response: SourceCategoryGroupedTags =
            self.reqwest_client.get(&url).send().await?.json().await?;

        Ok(response)
    }

    pub async fn get_media_sources(&self, hash: &str) -> Result<Vec<MediaSource>> {
        let url = format!("{}/get_media_sources?hash={}", self.base_url, hash);
        let response: Vec<MediaSource> = self.reqwest_client.get(&url).send().await?.json().await?;

        Ok(response)
    }

    pub async fn set_media_favorite(&self, hash: &str, is_favorite: bool) -> Result<()> {
        let url = format!(
            "{}/set_media_favorite?hash={}&is_favorite={}",
            self.base_url, hash, is_favorite
        );
        let _ = self.reqwest_client.put(&url).send().await?;

        Ok(())
    }

    pub async fn get_video_length(&self, hash: &str) -> Result<Option<f64>> {
        let url = format!("{}/get_video_length?hash={}", self.base_url, hash);
        let response: Option<f64> = self.reqwest_client.get(&url).send().await?.json().await?;

        Ok(response)
    }

    pub async fn get_top_n_closest_for_media(
        &self,
        hash: &str,
        n: i64,
    ) -> Result<Vec<EmbeddingDistance>> {
        let url = format!(
            "{}/get_top_n_closest_for_media?hash={}&n={}",
            self.base_url, hash, n
        );
        let response: Vec<EmbeddingDistance> =
            self.reqwest_client.get(&url).send().await?.json().await?;

        Ok(response)
    }

    pub async fn search(&self, query: &SearchCriteria) -> Result<Vec<Media>> {
        let url = format!("{}/search", self.base_url);
        let response: Vec<Media> = self
            .reqwest_client
            .post(&url)
            .json(query)
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn update_tags(&self, raw_input: &str, hash: &str) -> Result<()> {
        let url = format!(
            "{}/update_tags?raw_input={}&hash={}",
            self.base_url, raw_input, hash
        );
        let _ = self.reqwest_client.put(&url).send().await?;

        Ok(())
    }

    pub async fn delete_tags(&self, hash: &str, tags: Vec<String>) -> Result<()> {
        let tags_query = tags
            .into_iter()
            .map(|tag| format!("tags={}", tag))
            .collect::<Vec<String>>()
            .join("&");

        let url = format!("{}/delete_tags?hash={}&{}", self.base_url, hash, tags_query);
        let _ = self.reqwest_client.delete(&url).send().await?;

        Ok(())
    }

    pub async fn get_tags_as_text(&self, hash: &str) -> Result<Option<String>> {
        let url = format!("{}/get_tags_as_text?hash={}", self.base_url, hash);
        let response: Option<String> = self.reqwest_client.get(&url).send().await?.json().await?;

        Ok(response)
    }

    pub async fn get_list_of_all_tags_with_details(
        &self,
        ordering_criteria: AllTagsOrderingCriteria,
    ) -> Result<Vec<TagWithCount>> {
        let url = format!(
            "{}/get_list_of_all_tags_with_details?ordering_criteria={}",
            self.base_url, ordering_criteria
        );
        let response: Vec<TagWithCount> =
            self.reqwest_client.get(&url).send().await?.json().await?;

        Ok(response)
    }

    pub fn url(&self) -> String {
        self.base_url.clone()
    }
}
