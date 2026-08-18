use libsqlite3_sys::sqlite3_auto_extension;
use log::{error, info};
use sqlite_vec::sqlite3_vec_init;
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions};

use crate::config::global_config::GlobalConfig;

/// Gets the db paths from config, creates the dbs if they don't exist, runs any pending migrations
pub async fn prepare_dbs(main_db_path: &str, thumbs_db_path: &str) {
    prepare_main_db(main_db_path).await;
    prepare_thumbs_db(thumbs_db_path).await;
}

pub async fn prepare_dbs_from_config(config: &GlobalConfig) {
    prepare_main_db(&config.db.db_path).await;
    prepare_thumbs_db(&config.thumbs.thumbs_db_path).await;
}
pub async fn prepare_main_db(db_path: &str) {
    let db_path_absolute = std::path::absolute(db_path)
        .unwrap()
        .to_string_lossy()
        .to_string();

    info!("db_path: {}", &db_path_absolute);

    let does_db_exist = sqlx::Sqlite::database_exists(&db_path_absolute)
        .await
        .unwrap();

    if !does_db_exist {
        if db_path.is_empty() {
            error!("db_path is empty");
            return;
        }
        info!(
            "kasa database doesn't exist creating database at {}",
            db_path
        );
        sqlx::Sqlite::create_database(&db_path_absolute)
            .await
            .unwrap();
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(6)
        .connect(&db_path_absolute)
        .await
        .unwrap();

    info!("running main db migrations");
    sqlx::migrate!("../../migrations/db")
        .run(&pool)
        .await
        .unwrap();
}

pub async fn prepare_thumbs_db(db_path: &str) {
    let thumbs_path_absolute = std::path::absolute(db_path)
        .unwrap()
        .to_string_lossy()
        .to_string();

    info!("checking if thumbs db exists");

    let does_thumbs_db_exist = sqlx::Sqlite::database_exists(&thumbs_path_absolute)
        .await
        .unwrap();

    if !does_thumbs_db_exist {
        if db_path.is_empty() {
            error!("thumbs_db_path is empty");
            return;
        }
        info!(
            "thumbs database doesn't exist creating database at {}",
            db_path
        );
        sqlx::Sqlite::create_database(&thumbs_path_absolute)
            .await
            .unwrap();
    } else {
        info!("thumbs db exists skipping db creation");
        info!("thumbs db exists at {}", &thumbs_path_absolute);
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(6)
        .connect(&thumbs_path_absolute)
        .await
        .unwrap();

    info!("running thumbs db migrations");
    sqlx::migrate!("../../migrations/thumbs")
        .run(&pool)
        .await
        .unwrap();
}

pub unsafe fn init_sqlite_vec0() {
    unsafe {
        sqlite3_auto_extension(Some(unsafe {
            std::mem::transmute(sqlite3_vec_init as *const ())
        }))
    };
}
