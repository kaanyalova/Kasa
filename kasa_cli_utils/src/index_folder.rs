use kasa_core::index::indexer::index;
use sqlx::sqlite::SqlitePoolOptions;

use crate::IndexFolderArgs;

pub async fn index_folder(args: IndexFolderArgs) {
    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect(args.db_path.to_str().unwrap())
        .await
        .unwrap();

    let pool_thumbs = SqlitePoolOptions::new()
        .max_connections(8)
        .connect(args.thumbs_db_path.to_str().unwrap())
        .await
        .unwrap();

    index(args.folder.to_str().unwrap().into(), pool, pool_thumbs).await;
}
