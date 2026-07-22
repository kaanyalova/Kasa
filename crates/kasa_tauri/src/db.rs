use libsqlite3_sys::sqlite3_auto_extension;
use log::{error, info, trace};
use sqlite_vec::sqlite3_vec_init;
use tokio::sync::Mutex;

use kasa_core::{
    config::global_config::get_config_impl,
    db::{
        TagQueryOutput,
        db_info::{ThumbsDBInfo, get_thumbs_db_info_impl},
        migrations::{prepare_main_db, prepare_thumbs_db},
        query_tags_impl,
        schema::Media,
    },
    layout::google_photos::{ImageRow, MediaLayoutData, calculate_layout},
};
use sqlx::{
    Pool, Sqlite, query,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::{path::PathBuf, str::FromStr};
use tauri::{AppHandle, Manager};

use crate::{
    downloaders::{DownloaderState, DownloaderStore},
    remote_client::RemoteClient,
};

impl Default for DbStore {
    fn default() -> Self {
        DbStore::Local(LocalDbStore::default())
    }
}

pub enum DbStore {
    // todo add a third uninitialized type here
    Local(LocalDbStore),
    Remote(RemoteDbStore),
}

#[derive(Default)]
pub struct LocalDbStore {
    pub db: Mutex<Option<Pool<Sqlite>>>,
    pub thumbs_db: Mutex<Option<Pool<Sqlite>>>,
}

#[derive(Default)]
pub struct RemoteDbStore {
    pub client: RemoteClient,
    pub thumbs_db: Mutex<Option<Pool<Sqlite>>>,
}

#[derive(Default)]
pub struct MediaCache {
    pub media: Mutex<Option<Vec<Media>>>,
}

#[derive(Default)]
pub struct DatabaseState(pub Mutex<DbStore>);

#[tauri::command(async)]
#[specta::specta]
pub async fn query_tags(tag_name: String, count: i64, handle: AppHandle) -> Vec<TagQueryOutput> {
    println!("querying tags!");
    let state = handle.state::<DatabaseState>();
    let connection_state = state.0.lock().await;

    match &*connection_state {
        DbStore::Local(db_store) => {
            let guard = db_store.db.lock().await;

            if let Some(pool) = guard.as_ref() {
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
    }
}

#[tauri::command(async)]
#[specta::specta]

pub async fn are_dbs_mounted(handle: AppHandle) -> bool {
    let state = handle.state::<DatabaseState>();
    let connection_state = state.0.lock().await;

    match &*connection_state {
        DbStore::Local(db_store) => {
            let db_guard = db_store.db.lock().await;
            let thumbs_guard = db_store.thumbs_db.lock().await;
            db_guard.as_ref().is_some() && thumbs_guard.as_ref().is_some()
        }
        DbStore::Remote(remote_store) => {
            let response = remote_store.client.ping().await;

            if let Ok(resp) = response
                && resp == "pong"
            {
                return true;
            }

            false
        }
    }
}

#[tauri::command(async)]
#[specta::specta]
/// Mounts the dbs into db_store, runs any pending migrations
pub async fn connect_dbs(handle: AppHandle) {
    let config = get_config_impl();

    unsafe {
        sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
    }

    prepare_thumbs_db(&config.thumbs.thumbs_db_path).await;

    let thumbs_path_absolute = std::path::absolute(&config.thumbs.thumbs_db_path)
        .unwrap()
        .to_string_lossy()
        .to_string();

    let thumbs_options = SqliteConnectOptions::from_str(&thumbs_path_absolute)
        .unwrap()
        .pragma("journal_mode", "WAL")
        .pragma("synchronous", "NORMAL");

    let pool_thumbs = SqlitePoolOptions::new()
        .max_connections(32)
        .connect_with(thumbs_options)
        .await
        .unwrap();

    // mount the dbs
    let handle_db = handle.clone();
    let state = handle_db.state::<DatabaseState>();
    let mut db_store = state.0.lock().await;

    let is_server =
        config.db.db_path.starts_with("http://") || config.db.db_path.starts_with("https://");

    let handle_downloader = handle.clone();

    if is_server {
        let client = RemoteClient::new(config.db.db_path.clone());
        *db_store = DbStore::Remote(RemoteDbStore {
            client,
            thumbs_db: Mutex::new(Some(pool_thumbs)),
        });

        // create the downloader stuff here
        let downloader_state = handle.state::<DownloaderState>();
        let mut downloader_store = downloader_state.0.lock().await;

        // todo this breaks the whole app if the websockets fails, handle it more gracefully
        let remote_downloader = DownloaderStore::new_remote(handle_downloader, &config)
            .await
            .unwrap();

        *downloader_store = remote_downloader;
    }
    // local db
    else {
        prepare_main_db(&config.db.db_path).await;

        let db_path_absolute = std::path::absolute(&config.db.db_path)
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

        let pool_downloader = pool_db.clone();
        let pool_thumbs_downloader = pool_thumbs.clone();

        *db_store = DbStore::Local(LocalDbStore {
            db: Mutex::new(Some(pool_db)),
            thumbs_db: Mutex::new(Some(pool_thumbs)),
        });

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
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn does_the_db_file_exist(handle: AppHandle) -> bool {
    let state = handle.state::<DatabaseState>();
    let connection_state = state.0.lock().await;

    let config = get_config_impl();
    if config.db.db_path.starts_with("http://") || config.db.db_path.starts_with("https://") {
        return true;
    }

    match &*connection_state {
        DbStore::Local(_local_db_store) => {
            let config = get_config_impl();
            PathBuf::from(config.db.db_path).exists()
        }
        DbStore::Remote(_remote_db_store) => true,
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
    let state = handle.state::<DatabaseState>();
    let connection_state = state.0.lock().await;

    match &*connection_state {
        DbStore::Local(db_store) => {
            let guard = db_store.thumbs_db.lock().await;
            if let Some(pool) = guard.as_ref() {
                Some(get_thumbs_db_info_impl(pool).await)
            } else {
                None
            }
        }
        DbStore::Remote(remote_store) => {
            let guard = remote_store.thumbs_db.lock().await;
            if let Some(pool) = guard.as_ref() {
                Some(get_thumbs_db_info_impl(pool).await)
            } else {
                None
            }
        }
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn nuke_db_versioning(handle: AppHandle) {
    let state = handle.state::<DatabaseState>();
    let connection_state = state.0.lock().await;

    match &*connection_state {
        DbStore::Local(db_store) => {
            let guard = db_store.thumbs_db.lock().await;
            if let Some(pool) = guard.as_ref() {
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
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn is_remote_db(handle: AppHandle) -> bool {
    let state = handle.state::<DatabaseState>();
    let connection_state = state.0.lock().await;

    matches!(&*connection_state, DbStore::Remote(_))
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_remote_server_url(handle: AppHandle) -> String {
    let state = handle.state::<DatabaseState>();
    let connection_state = state.0.lock().await;

    match &*connection_state {
        DbStore::Local(_) => {
            error!("get_remote_media_url called on local db");
            "".to_string()
        }
        DbStore::Remote(remote_store) => remote_store.client.url(),
    }
}
