use log::{error, info, trace};
use tauri_specta::Event;
use tokio::sync::Mutex;

use kasa_core::config::global_config::DatabaseType;
use kasa_core::{
    config::global_config::{get_config_impl, set_db_type},
    db::{
        TagQueryOutput,
        db_info::{ThumbsDBInfo, get_thumbs_db_info_impl},
        migrations::{init_sqlite_vec0, prepare_main_db, prepare_thumbs_db},
        query_tags_impl,
        schema::Media,
    },
    layout::google_photos::{ImageRow, MediaLayoutData, calculate_layout},
};
use sqlx::{
    Pool, Sqlite, query,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};
use tauri::{App, AppHandle, Manager};

use crate::{
    config::set_db_path,
    downloaders::{DownloaderState, DownloaderStore},
    events::DatabaseConnectionEvent,
    remote_client::RemoteClient,
};

#[derive(Clone)]
pub enum DbStore {
    WaitingForFrontend,
    Local(LocalDbStore),
    Remote(RemoteDbStore),
    Uninitialized,
    Errored(String),
}

impl DbStore {
    pub async fn connect_to_remote_db(handle: AppHandle, url: String) -> DbStore {
        let config = get_config_impl();
        let client = RemoteClient::new(&url);
        let thumbs_db = Self::connect_to_thumbs_db().await;

        let remote_downloader = DownloaderStore::new_remote(handle.clone(), &config).await;

        let remote_downloader = match remote_downloader {
            Ok(r) => r,
            Err(e) => {
                return DbStore::errored(&e.to_string());
            }
        };

        let downloader_store = handle.state::<DownloaderState>();
        let mut locked = downloader_store.0.lock().await;

        *locked = remote_downloader;

        DbStore::Remote(RemoteDbStore {
            client,
            thumbs_db: Some(thumbs_db),
        })
    }

    /// Create a new local db and connect to it
    pub async fn connect_to_new_local_db(handle: AppHandle, path: String) -> DbStore {
        prepare_main_db(&path).await;

        let db_path_absolute = std::path::absolute(&path)
            .unwrap()
            .to_string_lossy()
            .to_string();

        let db_options = SqliteConnectOptions::from_str(&db_path_absolute)
            .unwrap()
            .pragma("journal_mode", "WAL")
            .pragma("synchronous", "NORMAL");

        let pool_db = SqlitePoolOptions::new()
            .max_connections(32)
            .connect_with(db_options)
            .await
            .unwrap();

        let pool_thumbs = Self::connect_to_thumbs_db().await;

        let config = get_config_impl();

        let pool_downloader = pool_db.clone();
        let pool_thumbs_downloader = pool_thumbs.clone();

        // if we are here, i assume the dbs are mounted properly, load the downloader
        let handle_downloader = handle.clone();
        let local_downloader = DownloaderStore::new_local(
            handle_downloader,
            pool_downloader,
            pool_thumbs_downloader,
            &config,
        )
        .await
        .unwrap();

        let downloader_state = handle.state::<DownloaderState>();
        {
            let mut downloader_store = downloader_state.0.lock().await;
            *downloader_store = local_downloader;
        }

        DbStore::Local(LocalDbStore {
            db: Some(pool_db),
            thumbs_db: Some(pool_thumbs),
        })
    }
    /// Connect to an existing local db, errors out if the db file doesn't exit
    pub async fn connect_to_existing_local_db(handle: AppHandle, path: String) -> DbStore {
        let path_converted = Path::new(&path);
        if !path_converted.exists() {
            return DbStore::errored(&format!(
                "The db at {} doesn't exist.",
                path_converted.to_string_lossy().to_string()
            ));
        }

        Self::connect_to_new_local_db(handle.clone(), path).await
    }

    pub fn errored(message: &str) -> DbStore {
        DbStore::Errored(message.to_string())
    }

    async fn emit_update_event(handle: AppHandle) {
        let db_state = handle.state::<DatabaseState>();
        let store = db_state.clone_store().await; // locks internally, drops guard

        let event_option = match store {
            DbStore::Errored(e) => Some(DatabaseConnectionEvent::Failed(e)), // no clone needed now
            DbStore::Local(_) => Some(DatabaseConnectionEvent::LocalConnected),
            DbStore::Remote(_) => Some(DatabaseConnectionEvent::RemoteConnected),
            DbStore::Uninitialized => Some(DatabaseConnectionEvent::Uninitialize),
            DbStore::WaitingForFrontend => None,
        };

        if let Some(e) = event_option {
            e.emit(&handle).unwrap();
        }
    }

    async fn connect_to_thumbs_db() -> Pool<Sqlite> {
        let config = get_config_impl();

        prepare_thumbs_db(&config.thumbs.thumbs_db_path).await;

        let thumbs_path_absolute = std::path::absolute(&config.thumbs.thumbs_db_path)
            .unwrap()
            .to_string_lossy()
            .to_string();

        let thumbs_options = SqliteConnectOptions::from_str(&thumbs_path_absolute)
            .unwrap()
            .pragma("journal_mode", "WAL")
            .pragma("synchronous", "NORMAL");

        SqlitePoolOptions::new()
            .max_connections(32)
            .connect_with(thumbs_options)
            .await
            .unwrap()
    }
}

#[derive(Default, Clone)]
pub struct LocalDbStore {
    pub db: Option<Pool<Sqlite>>,
    pub thumbs_db: Option<Pool<Sqlite>>,
}

#[derive(Default, Clone)]
pub struct RemoteDbStore {
    pub client: RemoteClient,
    pub thumbs_db: Option<Pool<Sqlite>>,
}

#[derive(Default)]
pub struct MediaCache {
    pub media: Mutex<Option<Vec<Media>>>,
}

pub struct DatabaseState(pub Mutex<DbStore>);

impl DatabaseState {
    pub fn wait_for_frontend() -> Self {
        Self(Mutex::new(DbStore::WaitingForFrontend))
    }
}

impl DatabaseState {
    /// Clones the current store out from under the lock so commands can do their
    /// work (including network requests and long queries) without holding the mutex.
    /// The clone is cheap: `Pool` is an `Arc`, `RemoteClient` is an `Arc` + `String`.
    pub async fn clone_store(&self) -> DbStore {
        self.0.lock().await.clone()
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn query_tags(tag_name: String, count: i64, handle: AppHandle) -> Vec<TagQueryOutput> {
    println!("querying tags!");
    let db_store = handle.state::<DatabaseState>().clone_store().await;

    match db_store {
        DbStore::Local(db_store) => {
            if let Some(pool) = db_store.db.as_ref() {
                query_tags_impl(tag_name, count, pool).await
            } else {
                error!("no db found when querying tags");
                vec![]
            }
        }
        DbStore::Remote(remote_db_store) => remote_db_store
            .client
            .query_tags(&tag_name, count)
            .await
            .unwrap(),
        _ => panic!("db not initialized"),
    }
}

async fn connect_to_db_impl(
    handle: AppHandle,
    path: &str,
    create_new_db: bool,
    db_type: DatabaseType,
) {
    let db_store = if path == "" {
        DbStore::Uninitialized
    } else {
        match db_type {
            DatabaseType::Remote => {
                DbStore::connect_to_remote_db(handle.clone(), path.to_string()).await
            }
            DatabaseType::Local => {
                if create_new_db {
                    DbStore::connect_to_new_local_db(handle.clone(), path.to_string()).await
                } else {
                    DbStore::connect_to_existing_local_db(handle.clone(), path.to_string()).await
                }
            }
            DatabaseType::Unknown => {
                let failed_text =
                    "The database_type field of the config must either be \"local\" or \"remote\" "
                        .to_string();

                DbStore::Errored(failed_text)
            }
        }
    };

    let db_state = handle.state::<DatabaseState>();
    let mut locked = db_state.0.lock().await;
    *locked = db_store;
}

#[tauri::command(async)]
#[specta::specta]
pub async fn connect_to_new_local_db(handle: AppHandle, path: String) {
    set_db_path(&path);
    connect_to_db_impl(handle.clone(), &path, true, DatabaseType::Local).await;
    DbStore::emit_update_event(handle).await;
}

#[tauri::command(async)]
#[specta::specta]
pub async fn connect_to_existing_local_db(handle: AppHandle, path: String) {
    set_db_path(&path);
    set_db_type(DatabaseType::Local);
    connect_to_db_impl(handle.clone(), &path, false, DatabaseType::Local).await;
    DbStore::emit_update_event(handle).await;
}
#[tauri::command(async)]
#[specta::specta]
pub async fn connect_to_remote_db(handle: AppHandle, url: String) {
    set_db_path(&url);
    set_db_type(DatabaseType::Remote);
    connect_to_db_impl(handle.clone(), &url, false, DatabaseType::Remote).await;
    DbStore::emit_update_event(handle).await;
}

#[tauri::command(async)]
#[specta::specta]
pub async fn connect_to_db_in_config(handle: AppHandle) {
    let config = get_config_impl();
    let db_path = config.db.db_path;
    let db_type = config.db.db_type;

    connect_to_db_impl(handle.clone(), &db_path, false, db_type).await;
    DbStore::emit_update_event(handle).await;
}

#[tauri::command(async)]
#[specta::specta]
pub async fn does_the_db_file_exist(handle: AppHandle) -> bool {
    let db_store = handle.state::<DatabaseState>().clone_store().await;

    let config = get_config_impl();
    if config.db.db_path.starts_with("http://") || config.db.db_path.starts_with("https://") {
        return true;
    }

    match db_store {
        DbStore::Local(_local_db_store) => {
            let config = get_config_impl();
            PathBuf::from(config.db.db_path).exists()
        }
        DbStore::Remote(_remote_db_store) => true,
        _ => panic!("db not initialized"),
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_layout_from_cache(
    handle: AppHandle,
    width: f64,
    gaps: u64,
    scale: f64,
) -> Option<Vec<ImageRow>> {
    let cache = handle.state::<MediaCache>().media.lock().await.clone(); // TODO: lots of clones here , somehow remove them?
    trace!(
        "doing layout work for {} items",
        cache.as_ref().map(|c| c.len()).unwrap_or(0)
    );

    if let Some(media) = cache {
        let layout_data = media
            .into_iter()
            .map(|m| MediaLayoutData {
                hash: m.hash,
                thumbnail_x: m.thumbnail_x as i64,
                thumbnail_y: m.thumbnail_y as i64,
            })
            .collect();
        Some(calculate_layout(layout_data, scale, width, gaps))
    } else {
        info!("No media found on cache!");
        None
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_thumbs_db_info(handle: AppHandle) -> Option<ThumbsDBInfo> {
    let db_store = handle.state::<DatabaseState>().clone_store().await;

    match db_store {
        DbStore::Local(db_store) => {
            if let Some(pool) = db_store.thumbs_db.as_ref() {
                Some(get_thumbs_db_info_impl(pool).await)
            } else {
                None
            }
        }
        DbStore::Remote(remote_store) => {
            if let Some(pool) = remote_store.thumbs_db.as_ref() {
                Some(get_thumbs_db_info_impl(pool).await)
            } else {
                None
            }
        }
        _ => panic!("db not initialized"),
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn nuke_db_versioning(handle: AppHandle) {
    let db_store = handle.state::<DatabaseState>().clone_store().await;

    match db_store {
        DbStore::Local(db_store) => {
            if let Some(pool) = db_store.thumbs_db.as_ref() {
                query("DROP TABLE _sqlx_migrations")
                    .execute(pool)
                    .await
                    .unwrap();
            } else {
                error!("Cannot connect to the db");
            }
        }
        DbStore::Remote(_) => {
            error!("dont do that!");
        }
        _ => panic!("db not initialized"),
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn is_remote_db(handle: AppHandle) -> bool {
    let db_store = handle.state::<DatabaseState>().clone_store().await;

    matches!(db_store, DbStore::Remote(_))
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_remote_server_url(handle: AppHandle) -> String {
    let db_store = handle.state::<DatabaseState>().clone_store().await;

    match db_store {
        DbStore::Local(_) => {
            error!("get_remote_media_url called on local db");
            "".to_string()
        }
        DbStore::Remote(remote_store) => remote_store.client.url(),
        _ => panic!("db not initialized"),
    }
}
