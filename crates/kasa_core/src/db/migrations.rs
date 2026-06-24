use log::{error, info};
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions};

use crate::config::global_config::GlobalConfig;

/// Gets the db paths from config, creates the dbs if they don't exist, runs any pending migrations
pub async fn prepare_dbs(config: &GlobalConfig) {
    prepare_main_db(config).await;
    prepare_thumbs_db(config).await;
}

pub async fn prepare_main_db(config: &GlobalConfig) {
    let db_path_absolute = std::path::absolute(&config.db.db_path)
        .unwrap()
        .to_string_lossy()
        .to_string();

    info!("db_path: {}", &db_path_absolute);

    let does_db_exist = sqlx::Sqlite::database_exists(&db_path_absolute)
        .await
        .unwrap();

    if !does_db_exist {
        if config.db.db_path.is_empty() {
            error!("db_path is empty");
            return;
        }
        info!(
            "kasa database doesn't exist creating database at {}",
            &config.db.db_path
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

pub async fn prepare_thumbs_db(config: &GlobalConfig) {
    let thumbs_path_absolute = std::path::absolute(&config.thumbs.thumbs_db_path)
        .unwrap()
        .to_string_lossy()
        .to_string();

    info!("checking if thumbs db exists");

    let does_thumbs_db_exist = sqlx::Sqlite::database_exists(&thumbs_path_absolute)
        .await
        .unwrap();

    if !does_thumbs_db_exist {
        if config.thumbs.thumbs_db_path.is_empty() {
            error!("thumbs_db_path is empty");
            return;
        }
        info!(
            "thumbs database doesn't exist creating database at {}",
            &config.thumbs.thumbs_db_path
        );
        sqlx::Sqlite::create_database(&config.thumbs.thumbs_db_path)
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
