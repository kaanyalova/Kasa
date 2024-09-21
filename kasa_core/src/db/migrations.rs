use std::fs;

use log::{error, info, trace};
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions};

use crate::config::global_config::GlobalConfig;

/// Gets the db paths from config, creates the dbs if they don't exist, runs any pending migrations
pub async fn prepare_dbs(config: &GlobalConfig) {
    // WARNING ON DEVELOPMENT this causes different path outputs when using the cli and
    // the tauri app, tauri seems to have ./kasa_tauri as its base directory while
    // kasa_cli_utils have ./ as its base dir. Don't use the cli without --db-path
    // if you have something like ../dev.kasa in your config.toml or it will create
    // the db at the parent dir of this repo
    let db_path_absolute = std::path::absolute(&config.db.db_path)
        .unwrap()
        .to_string_lossy()
        .to_string();

    let thumbs_path_absolute = std::path::absolute(&config.thumbs.thumbs_db_path)
        .unwrap()
        .to_string_lossy()
        .to_string();

    info!("trying to connect to dbs");
    //let config = get_config_impl();

    let does_db_exist = sqlx::Sqlite::database_exists(&db_path_absolute)
        .await
        .unwrap();

    if !does_db_exist {
        // check for empty paths in config
        if config.db.db_path == "" {
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

    let does_thumbs_db_exist = sqlx::Sqlite::database_exists(&thumbs_path_absolute)
        .await
        .unwrap();

    info!("checking if thumbs db exists");
    if !does_thumbs_db_exist {
        // check for empty paths in config
        if config.db.db_path == "" {
            error!("thumbs_db_path is empty");
            return;
        }
        // check for empty paths in config
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

    let pool_db = SqlitePoolOptions::new()
        .max_connections(6)
        .connect(&db_path_absolute)
        .await
        .unwrap();

    let pool_thumbs = SqlitePoolOptions::new()
        .max_connections(6)
        .connect(&thumbs_path_absolute)
        .await
        .unwrap();

    info!("running migrations");

    dbg!(db_path_absolute);
    // run migrations
    // TODO show that migrations are running to users
    sqlx::migrate!("../migrations/db")
        .run(&pool_db)
        .await
        .unwrap();

    dbg!(thumbs_path_absolute);
    sqlx::migrate!("../migrations/thumbs")
        .run(&pool_thumbs)
        .await
        .unwrap();
}
