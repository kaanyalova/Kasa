use std::env;

use config::get_config;
use config::set_config_resolution_value;
use config::set_config_value;
use config::set_db_path;
use config::set_thumbs_db_path;
use db::DbStore;
use db::MediaCache;
use db::are_dbs_mounted;
use db::connect_dbs;
use db::connect_to_db;
use db::get_layout_from_cache;
use db::get_thumbs_db_info;
use db::nuke_db_versioning;
use db::query_tags;
use downloaders::ExtractorsStore;
use downloaders::PythonStore;
use downloaders::download_and_index;
use file_picker::new_linux_file_picker_dialog_file_select;
use file_picker::new_linux_file_picker_dialog_multiple_folder_select;
use file_picker::new_linux_file_picker_dialog_save_file;
use file_picker::open_file_manager_with_file_selected;
use image::get_thumbnail;
use image::get_thumbnail_from_db;
use index::cleanup_unreferenced_files;
use index::index_path;
use index::nuke_all_indexes;
use index::nuke_selected_index;
use index::*;
use log::LevelFilter;
use log::warn;
use media::get_group_info;
use media::get_info;
use media::get_media_name;
use media::get_media_type;
use media::get_swf_resolution;
use media::get_tags;
use media::get_tags_grouped_by_source_categories;
use media_server::MediaServerStore;
use media_server::close_server;
use media_server::serve_media;
use search::SearchState;
use search::search;
use search::set_search_store;
use specta_typescript::BigIntExportBehavior;
use specta_typescript::Typescript;
use tags::delete_tags;
use tags::get_list_of_all_tags_with_details;
use tags::get_tags_as_text;
use tags::update_tags;
use tauri_specta::{Builder, collect_commands};
use utils::get_env_var;
use utils::image_path_to_rgba_bytes;
use utils::open_with_system_default_app;

mod db;
mod image;
mod media;
//mod serve_media;
mod config;
mod downloaders;
mod file_picker;
mod index;
mod media_server;
mod search;
mod tags;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // make vscode stop setting the GDK_BACKEND to x11 on wayland
    #[cfg(target_os = "linux")]
    unsafe {
        //std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        // needed for video player, browser dies otherwise
        // Updating webkitgtk seems to fix the brokenness
        // for now...

        if std::env::var("XDG_SESSION_TYPE") == Ok("wayland".to_string()) {
            std::env::set_var("GDK_BACKEND", "wayland");
        }

        std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    }

    let dotenv = dotenvy::dotenv();

    #[cfg(debug_assertions)]
    let default_log_level = LevelFilter::Debug;
    #[cfg(not(debug_assertions))]
    let default_log_level = LevelFilter::Warn;

    let log_level_env = env::var("KASA_LOG")
        .unwrap_or("".to_string())
        .to_lowercase();

    let log_level = match log_level_env.as_ref() {
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => default_log_level,
    };

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
            serve_media,
            close_server,
            get_media_type,
            add_index_source,
            remove_index_source,
            get_index_paths,
            index_all,
            download_and_index,
            index_path,
            image_path_to_rgba_bytes,
            open_with_system_default_app,
            new_linux_file_picker_dialog_multiple_folder_select,
            new_linux_file_picker_dialog_save_file,
            new_linux_file_picker_dialog_file_select,
            nuke_all_indexes,
            nuke_selected_index,
            cleanup_unreferenced_files,
            get_swf_resolution,
            get_group_info,
            delete_tags,
            get_tags_as_text,
            nuke_db_versioning,
            get_tags_grouped_by_source_categories,
            get_list_of_all_tags_with_details,
            open_file_manager_with_file_selected,
            set_search_store,
            set_db_path,
            set_thumbs_db_path,
            get_media_name
        ]
    });

    #[cfg(all(not(target_os = "android"), debug_assertions))]
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
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_log::Builder::new().level(log_level).build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_fs::init())
        //.plugin(tauri_plugin_theme::init(context.config_mut()))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            Ok(())
        })
        .manage(DbStore::default())
        .manage(MediaCache::default())
        .manage(MediaServerStore::default())
        .manage(PythonStore::default())
        .manage(ExtractorsStore::default())
        .manage(SearchState::default())
        .run(context)
        .expect("error while running tauri application");
}
