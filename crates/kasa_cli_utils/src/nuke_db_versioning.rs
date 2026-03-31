use kasa_core::config::global_config::get_config_impl;
use sqlx::query;

pub async fn nuke_db_versioning() {
    let config = get_config_impl();

    let db = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&config.db.db_path)
        .await
        .unwrap();

    query("DROP TABLE IF EXISTS _sqlx_migrations")
        .execute(&db)
        .await
        .unwrap();
}
