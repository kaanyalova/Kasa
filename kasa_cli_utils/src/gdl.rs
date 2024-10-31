use kasa_core::{
    config::global_config::get_config_impl, db::migrations::prepare_dbs,
    downloaders::gallery_dl::download_and_index,
};
use kasa_python::init_interpreter;
use sqlx::sqlite::SqlitePoolOptions;

pub async fn gdl(url: &str) {
    let config = get_config_impl();
    let interpreter = init_interpreter();

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

    download_and_index(
        interpreter,
        url,
        &config.downloader.output_path,
        &pool,
        &pool_thumbs,
    )
    .await
    .unwrap();
}
