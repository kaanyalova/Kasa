use kasa_core::{
    config::global_config::{GlobalConfig, get_config_impl},
    db::migrations::prepare_dbs,
    index::indexer::index,
};
use sqlx::sqlite::SqlitePoolOptions;

use crate::IndexFolderArgs;

pub async fn index_folder(args: IndexFolderArgs) {
    let mut config = {
        if args.use_config_file {
            get_config_impl()
        } else {
            GlobalConfig::default()
        }
    };

    // force db paths when no config file is used
    if !args.use_config_file {
        assert!(
            args.db_path.is_some(),
            "You need to provide --thumbs-db-path and --db-path when not using config file"
        );
        assert!(
            args.thumbs_db_path.is_some(),
            "You need to provide --thumbs-db-path and --db-path when not using config file"
        );
    }

    if let Some(path) = args.db_path {
        config.db.db_path = path.display().to_string();
    }

    if let Some(path) = args.thumbs_db_path {
        config.thumbs.thumbs_db_path = path.display().to_string();
    }

    prepare_dbs(&config).await;
    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect(&config.db.db_path)
        .await
        .unwrap();

    let pool_thumbs = SqlitePoolOptions::new()
        .max_connections(8)
        .connect(&config.thumbs.thumbs_db_path)
        .await
        .unwrap();

    index(args.folder.to_str().unwrap(), &pool, &pool_thumbs).await;
}
