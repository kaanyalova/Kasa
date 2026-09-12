use std::{
    path::{self, Path},
    str::FromStr,
    sync::Mutex,
};
mod database;
mod downloader;

use crate::events::{CacheUpdatedEvent, KasaEvent, TagsUpdatedEvent};
use crate::{
    clients::{
        EventCallback, KasaClient,
        database::DbStore,
        downloader::DownloaderStore,
        local::{database::LocalDb, downloader::LocalDownloader},
    },
    errors::ClientError,
};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use kasa_core::{
    config::global_config::GlobalConfig,
    db::{
        TagQueryOutput,
        embeddings::{EmbeddingDistance, get_top_n_closest_for_media_impl},
        migrations::prepare_dbs,
        query_tags_impl,
        schema::{Media, MediaSource},
    },
    downloaders::download_queue::DownloaderStateUpdate,
    media::{
        MediaInfo, SourceCategoryGroupedTags, TagWithDetails, get_info_impl, get_media_name_impl,
        get_media_sources_impl, get_media_type_impl, get_tags_detailed_impl,
        get_tags_grouped_by_source_categories_impl, get_valid_path_impl, get_video_length_impl,
        set_media_favorite_impl,
    },
    tags::{
        AllTagsOrderingCriteria, TagWithCount, get_list_of_all_tags_with_details_impl,
        get_tags_as_text_impl, remove_tags, search::SearchCriteria, update_tags_impl,
    },
    thumbnail::thumbnailer::generate_or_get_thumbnail_from_db_impl,
};
use sqlx::{
    Pool, Sqlite,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use tokio::sync::{Mutex as AsyncMutex, mpsc};

struct LocalKasaClient {
    pub database: Mutex<DbStore<LocalDb>>,
    pub downloader: AsyncMutex<DownloaderStore<LocalDownloader>>,
    pub on_event: Option<EventCallback>,
}

impl LocalKasaClient {
    pub fn wait_for_frontend() -> Self {
        Self {
            database: Mutex::new(DbStore::WaitingForFrontend),
            downloader: AsyncMutex::new(DownloaderStore::Uninitialized),
            on_event: None,
        }
    }

    pub async fn initialize(&mut self, config: &GlobalConfig) {
        match self.initialize_dbs(config).await {
            Ok(db) => {
                let mut guard = self.database.lock().unwrap();
                *guard = db;
            }
            Err(e) => {
                self.error_out(&e.to_string());
                return;
            }
        }

        match self.initialize_downloader(config).await {
            Ok(db) => {
                let mut guard = self.downloader.lock().await;
                *guard = db;
            }
            Err(e) => {
                self.error_out(&e.to_string());
                return;
            }
        }
    }

    pub fn error_out(&mut self, message: &str) {
        let mut guard = self.database.lock().unwrap();
        *guard = DbStore::Errored(message.to_owned());
    }

    async fn initialize_dbs(&self, config: &GlobalConfig) -> Result<DbStore<LocalDb>> {
        prepare_dbs(&config.db.db_path, &config.thumbs.thumbs_db_path).await;

        let pool_main = Self::connect_to_db(&config.db.db_path).await?;
        let pool_thumbs = Self::connect_to_db(&config.thumbs.thumbs_db_path).await?;

        Ok(DbStore::Initialized(LocalDb::new(pool_main, pool_thumbs)))
    }

    async fn initialize_downloader(
        &self,
        config: &GlobalConfig,
    ) -> Result<DownloaderStore<LocalDownloader>> {
        let dbs = self.get_dbs()?;

        let downloader = LocalDownloader::new(
            dbs.main_pool.clone(),
            dbs.thumbs_pool.clone(),
            config,
            self.on_event.clone(),
        )
        .await?;

        Ok(DownloaderStore::Initialized(downloader))
    }

    fn get_dbs(&self) -> Result<LocalDb> {
        let locked = self.database.lock().unwrap();

        if let DbStore::Initialized(db) = &*locked {
            return Ok(db.clone());
        }

        Err(ClientError::InvalidDb.into())
    }

    async fn connect_to_db(path: &str) -> Result<Pool<Sqlite>> {
        let absolute_path = path::absolute(path)?.to_string_lossy().to_string();

        let db_options = SqliteConnectOptions::from_str(&absolute_path)?
            .pragma("journal_mode", "WAL")
            .pragma("synchronous", "NORMAL");

        let pool_db = SqlitePoolOptions::new()
            .max_connections(32)
            .connect_with(db_options)
            .await
            .unwrap();

        Ok(pool_db)
    }
}

#[async_trait]
impl KasaClient for LocalKasaClient {
    fn call_event(&self, event: KasaEvent) {
        if let Some(on_event) = &self.on_event {
            on_event(event);
        }
    }

    fn set_event_handler(&mut self, callback: EventCallback) {
        self.on_event = Some(callback);
    }

    async fn get_info(&self, hash: &str) -> Result<Option<MediaInfo>> {
        let dbs = self.get_dbs()?;
        Ok(Some(get_info_impl(hash, &dbs.main_pool).await))
    }

    async fn get_tags(&self, hash: &str) -> Result<Vec<TagWithDetails>> {
        let dbs = self.get_dbs()?;
        Ok(get_tags_detailed_impl(hash, &dbs.main_pool).await)
    }

    async fn get_media_type(&self, hash: &str) -> Result<String> {
        let dbs = self.get_dbs()?;
        Ok(get_media_type_impl(hash, &dbs.main_pool).await)
    }

    async fn get_tags_grouped_by_source_categories(
        &self,
        hash: &str,
    ) -> Result<SourceCategoryGroupedTags> {
        let dbs = self.get_dbs()?;
        Ok(get_tags_grouped_by_source_categories_impl(hash, &dbs.main_pool).await)
    }

    async fn get_media_name(&self, hash: &str) -> Result<String> {
        let dbs = self.get_dbs()?;
        Ok(get_media_name_impl(hash, &dbs.main_pool).await)
    }

    async fn get_media_sources(&self, hash: &str) -> Result<Vec<MediaSource>> {
        let dbs = self.get_dbs()?;
        Ok(get_media_sources_impl(hash, &dbs.main_pool).await)
    }

    async fn set_media_favorite(&self, hash: &str, is_favorite: bool) -> Result<()> {
        let dbs = self.get_dbs()?;
        set_media_favorite_impl(hash, is_favorite, &dbs.main_pool).await;

        self.call_event(KasaEvent::CacheUpdated(CacheUpdatedEvent {
            reload_virtual_list: false,
        }));

        Ok(())
    }

    async fn get_video_length(&self, hash: &str) -> Result<Option<f64>> {
        let dbs = self.get_dbs()?;
        Ok(get_video_length_impl(hash, &dbs.main_pool).await)
    }

    async fn get_top_n_closest_for_media(
        &self,
        hash: &str,
        n: i64,
    ) -> Result<Vec<EmbeddingDistance>> {
        let dbs = self.get_dbs()?;
        Ok(get_top_n_closest_for_media_impl(&dbs.main_pool, hash, n).await?)
    }

    async fn get_valid_path(&self, hash: &str) -> Result<String> {
        let dbs = self.get_dbs()?;
        Ok(get_valid_path_impl(hash, &dbs.main_pool).await)
    }

    async fn query_tags(&self, query: &str, limit: i64) -> Result<Vec<TagQueryOutput>> {
        let dbs = self.get_dbs()?;
        Ok(query_tags_impl(query.to_string(), limit, &dbs.main_pool).await)
    }

    async fn get_thumbnail(&self, hash: &str) -> Result<Vec<u8>> {
        let dbs = self.get_dbs()?;
        let thumbnail =
            generate_or_get_thumbnail_from_db_impl(hash, &dbs.main_pool, &dbs.thumbs_pool).await;
        Ok(thumbnail.bytes)
    }

    async fn serve_media(&self, hash: &str) -> Result<Vec<u8>> {
        let dbs = self.get_dbs()?;

        let paths: Vec<String> = sqlx::query_scalar("SELECT path FROM Path WHERE hash = ?")
            .bind(hash)
            .fetch_all(&dbs.main_pool)
            .await?;

        let path = paths
            .into_iter()
            .find(|p| Path::new(p).exists())
            .ok_or_else(|| anyhow!("no file found for hash {hash}"))?;

        Ok(std::fs::read(path)?)
    }

    async fn search(&self, criteria: SearchCriteria) -> Result<Vec<Media>> {
        let dbs = self.get_dbs()?;
        let mut query = criteria.to_query();
        Ok(query.build_query_as().fetch_all(&dbs.main_pool).await?)
    }

    async fn update_tags(&self, raw_input: &str, hash: &str) -> Result<()> {
        let dbs = self.get_dbs()?;
        update_tags_impl(raw_input, hash.to_string(), &dbs.main_pool).await;

        self.call_event(KasaEvent::TagsUpdated(TagsUpdatedEvent {}));

        Ok(())
    }

    async fn delete_tags(&self, hash: &str, tags: &[String]) -> Result<()> {
        let dbs = self.get_dbs()?;
        remove_tags(tags.to_vec(), &dbs.main_pool, Some(hash.to_string())).await;

        self.call_event(KasaEvent::TagsUpdated(TagsUpdatedEvent {}));

        Ok(())
    }

    async fn get_tags_as_text(&self, hash: &str) -> Result<Option<String>> {
        let dbs = self.get_dbs()?;
        Ok(Some(get_tags_as_text_impl(hash, &dbs.main_pool).await))
    }

    async fn get_list_of_all_tags_with_details(
        &self,
        ordering_criteria: AllTagsOrderingCriteria,
    ) -> Result<Vec<TagWithCount>> {
        let dbs = self.get_dbs()?;
        Ok(get_list_of_all_tags_with_details_impl(&dbs.main_pool, ordering_criteria).await)
    }

    async fn push_download(&self, url: &str) -> Result<()> {
        let downloader = self.downloader.lock().await;
        if let DownloaderStore::Initialized(downloader) = &*downloader {
            downloader.queue_download(url).await;
        }
        Ok(())
    }
}
