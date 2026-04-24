use std::{
    sync::{Arc, Mutex},
    vec,
};

use kasa_core::{
    config::global_config::{get_config_impl, get_tag_extractors_dir},
    db::migrations::prepare_dbs,
    downloaders::gallery_dl::download_and_index_impl,
};
use kasa_python::{
    PyTrustMe,
    extractors::{
        TagExtractor,
        configurable::ConfigurableExtractor,
        scriptable::PythonTagExtractor,
    },
    init_interpreter, init_interpreter_with_gallery_dl,
};
use sqlx::sqlite::SqlitePoolOptions;

pub async fn gdl(url: &str) {
    let config = get_config_impl();
    let interpreter_gdl = Arc::new(PyTrustMe(init_interpreter_with_gallery_dl()));
    let interpreter_extractor = Arc::new(Mutex::new(PyTrustMe(init_interpreter())));

    let extractors_dir = get_tag_extractors_dir().unwrap();
    let python_extractor =
        PythonTagExtractor::init(interpreter_extractor.clone(), &extractors_dir).unwrap();
    let configurable_extractor = ConfigurableExtractor::init(&extractors_dir).unwrap();

    let extractors: Vec<&(dyn TagExtractor + Send + Sync)> =
        vec![&python_extractor, &configurable_extractor];

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

    download_and_index_impl(
        interpreter_gdl,
        url,
        &config.downloader.output_path,
        &pool,
        &pool_thumbs,
        &|_| {},
        |_| {},
        &extractors,
    )
    .await
    .unwrap();
}
