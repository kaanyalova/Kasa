use std::{
    path,
    str::FromStr,
    sync::{Arc, Mutex},
};
mod database;
mod downloader;
mod rest_client;

use crate::{
    clients::{
        EventCallback, KasaClient, KasaSharedMethods,
        database::DbStore,
        downloader::DownloaderStore,
        remote::{database::RemoteDb, downloader::RemoteDownloader, rest_client::RemoteClient},
    },
    errors::{ClientError, ClientResult as Result},
    events::{CacheUpdatedEvent, KasaEvent, TagsUpdatedEvent},
    runtime,
};
use async_trait::async_trait;
use kasa_core::{
    config::global_config::GlobalConfig,
    db::{
        TagQueryOutput,
        embeddings::EmbeddingDistance,
        migrations::prepare_thumbs_db,
        remote_cache::{
            get_media_name_from_remote_cache, get_media_type_from_remote_cache,
            get_video_length_from_remote_cache, insert_media_name_to_remote_cache,
            insert_media_type_to_remote_cache, insert_video_length_to_remote_cache,
        },
        schema::{Media, MediaSource, MediaType, media_type_to_string},
    },
    downloaders::download_queue::DownloaderStateUpdate,
    media::{MediaInfo, SourceCategoryGroupedTags, TagWithDetails},
    tags::{AllTagsOrderingCriteria, TagWithCount, search::SearchCriteria},
    thumbnail::{
        thumbnail_image::{Thumbnail, ThumbnailFormat},
        thumbnailer::{get_thumbnail_from_db_impl, insert_thumbnail_into_db_impl},
    },
};
use sqlx::{
    Pool, Sqlite,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use tokio::sync::{Mutex as AsyncMutex, mpsc};

pub struct RemoteKasaClient {
    pub database: Mutex<DbStore<RemoteDb>>,
    pub downloader: Arc<AsyncMutex<DownloaderStore<RemoteDownloader>>>,
    pub on_event: Option<EventCallback>,
}

impl RemoteKasaClient {
    pub fn wait_for_frontend() -> Self {
        Self {
            database: Mutex::new(DbStore::WaitingForFrontend),
            downloader: Arc::new(AsyncMutex::new(DownloaderStore::Uninitialized)),
            on_event: None,
        }
    }

    pub async fn initialize(&self, config: &GlobalConfig) {
        match self.initialize_impl(config).await {
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

    pub fn error_out(&self, message: &str) {
        let mut guard = self.database.lock().unwrap();
        *guard = DbStore::Errored(message.to_owned());
    }

    async fn initialize_impl(&self, config: &GlobalConfig) -> Result<DbStore<RemoteDb>> {
        prepare_thumbs_db(&config.thumbs.thumbs_db_path).await;

        let client = RemoteClient::new(&config.db.db_path);
        let pool_thumbs = Self::connect_to_db(&config.thumbs.thumbs_db_path).await?;

        Ok(DbStore::Initialized(RemoteDb::new(client, pool_thumbs)))
    }

    async fn initialize_downloader(
        &self,
        config: &GlobalConfig,
    ) -> Result<DownloaderStore<RemoteDownloader>> {
        let downloader = RemoteDownloader::new(config, self.on_event.clone()).await?;

        Ok(DownloaderStore::Initialized(downloader))
    }

    fn get_dbs(&self) -> Result<RemoteDb> {
        let locked = self.database.lock().unwrap();

        if let DbStore::Initialized(db) = &*locked {
            return Ok(db.clone());
        }

        Err(ClientError::InvalidDb)
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

    fn set_event_handler(&mut self, callback: EventCallback) {
        self.on_event = Some(callback);
    }
}

#[async_trait]
impl KasaClient for RemoteKasaClient {
    fn call_event(&self, event: KasaEvent) {
        if let Some(on_event) = &self.on_event {
            on_event(event);
        }
    }

    async fn get_info(&self, hash: &str) -> Result<Option<MediaInfo>> {
        let db = self.get_dbs()?;
        let hash = hash.to_owned();
        runtime()
            .spawn(async move { Ok(db.client.get_info(&hash).await?) })
            .await
            .unwrap()
    }

    async fn get_tags(&self, hash: &str) -> Result<Vec<TagWithDetails>> {
        let db = self.get_dbs()?;
        let hash = hash.to_owned();
        runtime()
            .spawn(async move { Ok(db.client.get_tags(&hash).await?) })
            .await
            .unwrap()
    }

    async fn get_media_type(&self, hash: &str) -> Result<String> {
        let db = self.get_dbs()?;
        let hash = hash.to_owned();
        runtime()
            .spawn(async move {
                if let Ok(Some(media_type)) =
                    get_media_type_from_remote_cache(&hash, &db.thumbs_pool).await
                {
                    return Ok(media_type_to_string(&media_type));
                }

                let media_type_str = db.client.get_media_type(&hash).await?;
                let media_type = media_type_str
                    .parse::<MediaType>()
                    .map_err(|e| ClientError::Anyhow(e.to_string()))?;
                insert_media_type_to_remote_cache(&hash, media_type, &db.thumbs_pool).await?;

                Ok(media_type_str)
            })
            .await
            .unwrap()
    }

    async fn get_tags_grouped_by_source_categories(
        &self,
        hash: &str,
    ) -> Result<SourceCategoryGroupedTags> {
        let db = self.get_dbs()?;
        let hash = hash.to_owned();
        runtime()
            .spawn(async move {
                Ok(db
                    .client
                    .get_tags_grouped_by_source_categories(&hash)
                    .await?)
            })
            .await
            .unwrap()
    }

    async fn get_media_name(&self, hash: &str) -> Result<String> {
        let db = self.get_dbs()?;
        let hash = hash.to_owned();
        runtime()
            .spawn(async move {
                if let Ok(Some(media_name)) =
                    get_media_name_from_remote_cache(&hash, &db.thumbs_pool).await
                {
                    return Ok(media_name);
                }

                let media_name = db.client.get_media_name(&hash).await?;
                insert_media_name_to_remote_cache(&hash, &media_name, &db.thumbs_pool).await?;

                Ok(media_name)
            })
            .await
            .unwrap()
    }

    async fn get_media_sources(&self, hash: &str) -> Result<Vec<MediaSource>> {
        let db = self.get_dbs()?;
        let hash = hash.to_owned();
        runtime()
            .spawn(async move { Ok(db.client.get_media_sources(&hash).await?) })
            .await
            .unwrap()
    }

    async fn set_media_favorite(&self, hash: &str, is_favorite: bool) -> Result<()> {
        let db = self.get_dbs()?;
        let hash = hash.to_owned();
        runtime()
            .spawn(async move { db.client.set_media_favorite(&hash, is_favorite).await })
            .await
            .unwrap()?;

        self.call_event(KasaEvent::CacheUpdated(CacheUpdatedEvent {
            reload_virtual_list: false,
        }));

        Ok(())
    }

    async fn get_video_length(&self, hash: &str) -> Result<Option<f64>> {
        let db = self.get_dbs()?;
        let hash = hash.to_owned();
        runtime()
            .spawn(async move {
                if let Ok(Some(video_length)) =
                    get_video_length_from_remote_cache(&hash, &db.thumbs_pool).await
                {
                    return Ok(Some(video_length));
                }

                let video_length = db.client.get_video_length(&hash).await?;
                if let Some(length) = video_length {
                    insert_video_length_to_remote_cache(&hash, length, &db.thumbs_pool).await?;
                }

                Ok(video_length)
            })
            .await
            .unwrap()
    }

    async fn get_top_n_closest_for_media(
        &self,
        hash: &str,
        n: i64,
    ) -> Result<Vec<EmbeddingDistance>> {
        let db = self.get_dbs()?;
        let hash = hash.to_owned();
        runtime()
            .spawn(async move { Ok(db.client.get_top_n_closest_for_media(&hash, n).await?) })
            .await
            .unwrap()
    }

    async fn get_valid_path(&self, hash: &str) -> Result<String> {
        let db = self.get_dbs()?;
        let hash = hash.to_owned();
        runtime()
            .spawn(async move { Ok(db.client.get_valid_path(&hash).await?) })
            .await
            .unwrap()
    }

    async fn query_tags(&self, query: &str, limit: i64) -> Result<Vec<TagQueryOutput>> {
        let db = self.get_dbs()?;
        let query = query.to_owned();
        runtime()
            .spawn(async move { Ok(db.client.query_tags(&query, limit).await?) })
            .await
            .unwrap()
    }

    async fn get_thumbnail(&self, hash: &str) -> Result<Vec<u8>> {
        let db = self.get_dbs()?;
        let hash = hash.to_owned();
        runtime()
            .spawn(async move {
                if let Some(thumbnail) = get_thumbnail_from_db_impl(&hash, &db.thumbs_pool).await {
                    if thumbnail.is_valid() {
                        return Ok(thumbnail.bytes);
                    }
                }

                let bytes = db.client.get_thumbnail(&hash).await?;

                if let Ok(image) = image::load_from_memory(&bytes) {
                    let format = match image::guess_format(&bytes) {
                        Ok(image::ImageFormat::Png) => ThumbnailFormat::Png,
                        Ok(image::ImageFormat::Jpeg) => ThumbnailFormat::Jpeg,
                        Ok(image::ImageFormat::Avif) => ThumbnailFormat::Avif,
                        Ok(image::ImageFormat::WebP) => ThumbnailFormat::WebpLossy,
                        _ => ThumbnailFormat::Png,
                    };

                    insert_thumbnail_into_db_impl(
                        &hash,
                        &Thumbnail {
                            x: image.width(),
                            y: image.height(),
                            bytes: bytes.clone(),
                            format,
                            success: true,
                        },
                        &db.thumbs_pool,
                    )
                    .await;
                }

                Ok(bytes)
            })
            .await
            .unwrap()
    }

    async fn serve_media(&self, hash: &str) -> Result<Vec<u8>> {
        let db = self.get_dbs()?;
        let hash = hash.to_owned();
        runtime()
            .spawn(async move { Ok(db.client.serve_media(&hash).await?) })
            .await
            .unwrap()
    }

    async fn search(&self, criteria: SearchCriteria) -> Result<Vec<Media>> {
        let db = self.get_dbs()?;
        runtime()
            .spawn(async move { Ok(db.client.search(&criteria).await?) })
            .await
            .unwrap()
    }

    async fn update_tags(&self, raw_input: &str, hash: &str) -> Result<()> {
        let db = self.get_dbs()?;
        let raw_input = raw_input.to_owned();
        let hash = hash.to_owned();
        runtime()
            .spawn(async move { db.client.update_tags(&raw_input, &hash).await })
            .await
            .unwrap()?;

        self.call_event(KasaEvent::TagsUpdated(TagsUpdatedEvent {}));

        Ok(())
    }

    async fn delete_tags(&self, hash: &str, tags: &[String]) -> Result<()> {
        let db = self.get_dbs()?;
        let hash = hash.to_owned();
        let tags = tags.to_vec();
        runtime()
            .spawn(async move { db.client.delete_tags(&hash, tags).await })
            .await
            .unwrap()?;

        self.call_event(KasaEvent::TagsUpdated(TagsUpdatedEvent {}));

        Ok(())
    }

    async fn get_tags_as_text(&self, hash: &str) -> Result<Option<String>> {
        let db = self.get_dbs()?;
        let hash = hash.to_owned();
        runtime()
            .spawn(async move { Ok(db.client.get_tags_as_text(&hash).await?) })
            .await
            .unwrap()
    }

    async fn get_list_of_all_tags_with_details(
        &self,
        ordering_criteria: AllTagsOrderingCriteria,
    ) -> Result<Vec<TagWithCount>> {
        let db = self.get_dbs()?;
        runtime()
            .spawn(async move {
                Ok(db
                    .client
                    .get_list_of_all_tags_with_details(ordering_criteria)
                    .await?)
            })
            .await
            .unwrap()
    }

    async fn push_download(&self, url: &str) -> Result<()> {
        let downloader = self.downloader.clone();
        let url = url.to_owned();
        runtime()
            .spawn(async move {
                let downloader = downloader.lock().await;
                if let DownloaderStore::Initialized(downloader) = &*downloader {
                    downloader.push_download(&url).await
                } else {
                    Ok(())
                }
            })
            .await
            .unwrap()?;
        Ok(())
    }

    fn load_config_from_path(&self, path: &str) -> GlobalConfig {
        KasaSharedMethods::load_config_from_path(path)
    }
    fn load_config_from_raw(&self, config_contents: &str) -> GlobalConfig {
        KasaSharedMethods::load_config_from_raw(config_contents)
    }
}
