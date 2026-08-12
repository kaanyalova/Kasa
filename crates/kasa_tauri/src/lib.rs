use std::env;

use tauri_specta::collect_events;
use tokio::sync::Mutex;

use crate::downloaders::{DownloaderState, DownloaderStore};
use crate::events::*;
use config::create_or_get_extractor_contents;
use config::create_or_get_path_for_extractor;
use config::get_config;
use config::get_example_metadata_for_extractor;
use config::get_existing_extractor_names;
use config::set_config_resolution_value;
use config::set_config_value_bool;
use config::set_config_value_f64;
use config::set_config_value_str;
use config::set_db_path;
use config::set_thumbs_db_path;
use db::MediaCache;
use db::are_dbs_mounted;
use db::connect_dbs;
use db::does_the_db_file_exist;
use db::get_layout_from_cache;
use db::get_remote_server_url;
use db::get_thumbs_db_info;
use db::is_remote_db;
use db::nuke_db_versioning;
use db::query_tags;
use downloaders::get_downloader_statuses;
use downloaders::queue_download_job;
use file_picker::new_linux_file_picker_dialog_file_select;
use file_picker::new_linux_file_picker_dialog_multiple_folder_select;
use file_picker::new_linux_file_picker_dialog_save_file;
use file_picker::open_file_manager_with_file_selected;
use image::get_thumbnail_from_db;
use index::cleanup_unreferenced_files;
use index::index_path;
use index::nuke_all_indexes;
use index::nuke_selected_index;
use index::*;
use kasa_python::GalleryDlStatus;
use log::LevelFilter;
use log::warn;
use media::get_group_info;
use media::get_info;
use media::get_media_name;
use media::get_media_sources;
use media::get_media_type;
use media::get_swf_resolution;
use media::get_tags;
use media::get_tags_grouped_by_source_categories;
use media::get_top_n_closest_for_media;
use media::get_video_length;
use media::set_media_favorite;
use media_server::MediaServerStore;
use media_server::close_server;
use media_server::serve_media;
use search::SearchState;
use search::search;
use search::set_search_criteria;
use search::set_search_input;
use specta_typescript::BigIntExportBehavior;
use specta_typescript::Typescript;
use tags::delete_tags;
use tags::get_list_of_all_tags_with_details;
use tags::get_tags_as_text;
use tags::update_tags;
use tauri::Manager;
use tauri_specta::{Builder, collect_commands};
use utils::get_env_var;
use utils::image_path_to_rgba_bytes;
use utils::open_with_system_default_app;

mod db;
mod image;
mod media;
//mod serve_media;
mod config;
pub mod downloaders;
mod events;
mod file_picker;
mod index;
mod media_server;
mod remote_client;
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

        //if std::env::var("XDG_SESSION_TYPE") == Ok("wayland".to_string()) {
        //    std::env::set_var("GDK_BACKEND", "wayland");
        //}

        // webkit is completely broken as of 2.50.1, especially if you are using nvidia,
        // https://bugs.webkit.org/show_bug.cgi?id=180739
        // https://bugs.webkit.org/buglist.cgi?quicksearch=WEBKIT_DISABLE_COMPOSITING_MODE
        // also broken on gnome-web scrolling fast just crashes the browser??? wtf?
        std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");

        // this at least makes it launch but the text is blurry and the canvas calculations are broken
        // (tag list doesn't work properly)
        //std::env::set_var("__NV_DISABLE_EXPLICIT_SYNC", "1");
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

    let builder = tauri_specta::Builder::<tauri::Wry>::new()
        .commands({
            collect_commands![
                query_tags,
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
                set_config_value_str,
                set_config_resolution_value,
                search,
                serve_media,
                close_server,
                get_media_type,
                add_index_source,
                remove_index_source,
                get_index_paths,
                index_all,
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
                set_search_criteria,
                set_search_input,
                set_db_path,
                set_thumbs_db_path,
                get_media_name,
                get_media_sources,
                set_media_favorite,
                get_video_length,
                set_config_value_bool,
                set_config_value_f64,
                set_config_value_str,
                queue_download_job,
                get_downloader_statuses,
                create_or_get_path_for_extractor,
                create_or_get_extractor_contents,
                get_existing_extractor_names,
                get_example_metadata_for_extractor,
                get_top_n_closest_for_media,
                does_the_db_file_exist,
                get_remote_server_url,
                is_remote_db,
            ]
        })
        .events(collect_events![
            DownloaderProgressUpdatedEvent,
            TagsUpdatedEvent,
            CacheUpdatedEvent,
            DbsUpdatedEvent,
            MediaServerDownEvent,
            OpenMediaModalEvent,
            CloseMediaModalEvent
        ]);

    #[cfg(all(not(target_os = "android"), debug_assertions))]
    let builder = {
        let builder = builder.typ::<GalleryDlStatus>();
        builder
            .export(
                // JS JSON.parse() cannot handle more than 2^52, and it doesn't convert to bigint
                Typescript::default().bigint(BigIntExportBehavior::Number),
                "../../src/lib/tauri_bindings.ts",
            )
            .unwrap();
        builder
    };

    #[cfg(not(all(not(target_os = "android"), debug_assertions)))]
    let builder = {
        use kasa_python::GalleryDlStatus;
        builder.typ::<GalleryDlStatus>()
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_log::Builder::new().level(log_level).build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_fs::init())
        //.plugin(tauri_plugin_theme::init(context.config_mut()))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(builder.invoke_handler())
        .manage(db::DatabaseState::default())
        .manage(MediaCache::default())
        .manage(MediaServerStore::default())
        .setup(move |app| {
            builder.mount_events(app);

            let handle = app.handle();
            app.manage(DownloaderState(Mutex::new(DownloaderStore::Uninitialized)));

            Ok(())
        })
        .manage(SearchState::default())
        .run(context)
        .expect("error while running tauri application");
}
