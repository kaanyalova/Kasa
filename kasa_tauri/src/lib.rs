use db::connect_to_db;
use db::get_layout_from_cache;
use db::query_all;
use db::query_tags;
use db::DbStore;
use db::MediaCache;
use image::get_thumbnail;
use linux::get_desktop;
use log::warn;
use media::get_info;
use media::get_tags;
//use serve_media::close_server;
//use serve_media::serve_media;
use config::get_config;
use config::set_config_resolution_value;
use config::set_config_value;
use db::are_dbs_mounted;
use db::connect_dbs;
use db::get_thumbs_db_info;
use image::get_thumbnail_from_db;
use search::search;
use specta_typescript::BigIntExportBehavior;
use specta_typescript::Typescript;
use tags::update_tags;
use utils::get_env_var;

use tauri_specta::{collect_commands, Builder};

mod db;
mod image;
mod linux;
mod media;
//mod serve_media;
mod config;
mod search;
mod tags;
mod utils;
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Its either broken images or bad performance ,amazing
    #[cfg(target_os = "linux")]
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");

    let dotenv = dotenvy::dotenv();

    match dotenv {
        Ok(_) => {
            warn!("A .env file was found, loading configuration from the file")
        }
        Err(_) => warn!("A .env was file not found, continuing..."),
    }

    let context = tauri::generate_context!();

    let builder = Builder::<tauri::Wry>::new().commands({
        collect_commands![
            connect_to_db,
            query_tags,
            query_all,
            get_desktop,
            get_thumbnail,
            get_info,
            get_layout_from_cache,
            update_tags,
            get_tags,
            get_env_var,
            are_dbs_mounted,
            get_config,
            connect_dbs,
            get_thumbnail_from_db,
            get_thumbs_db_info,
            set_config_value,
            set_config_resolution_value,
            search,
        ]
    });

    #[cfg(debug_assertions)]
    {
        builder
            .export(
                // JS JSON.parse() cannot handle more than 2^52, and it doesn't convert to bigint
                Typescript::default().bigint(BigIntExportBehavior::Number),
                "../src/lib/tauri_bindings.ts",
            )
            .unwrap();
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        //.plugin(tauri_plugin_theme::init(context.config_mut()))
        .plugin(tauri_plugin_os::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            Ok(())
        })
        .manage(DbStore::default())
        .manage(MediaCache::default())
        .run(context)
        .expect("error while running tauri application");
}
