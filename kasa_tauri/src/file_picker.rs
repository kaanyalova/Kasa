use std::{os::unix::process::CommandExt, process::Command};

use ashpd::{desktop::file_chooser::OpenFileRequest, zvariant::Str};
use axum::http::Uri;
use zbus::Connection;
use zbus_macros::proxy;
#[tauri::command]
#[specta::specta]
pub async fn new_linux_file_picker_dialog() -> Vec<String> {
    // This adds a few megabytes to the binary just for a proper file picker, tauri devs are refusing to upgrade their stuff
    // to gtk4
    #[cfg(target_os = "linux")]
    {
        let response = OpenFileRequest::default()
            .modal(true)
            .accept_label("Select")
            .multiple(true)
            .directory(true)
            .send()
            .await
            .unwrap()
            .response()
            .unwrap();

        response
            .uris()
            .into_iter()
            .map(|uri| uri.path().to_string())
            .collect()
    }

    #[cfg(not(target_os = "linux"))]
    vec![]
}

#[tauri::command(async)]
#[specta::specta]
pub async fn open_file_manager_with_file_selected(file_path: String) {
    #[cfg(target_os = "linux")]
    {
        #[proxy(
            default_service = "org.freedesktop.FileManager1",
            default_path = "/org/freedesktop/FileManager1"
        )]
        trait FileManager1 {
            #[allow(non_snake_case)]
            fn show_items(&self, URIs: Vec<String>, startup_id: String) -> zbus::Result<()>;
        }

        // https://unix.stackexchange.com/a/581215
        // dbus-send --session --print-reply --dest=org.freedesktop.FileManager1 --type=method_call /org/freedesktop/FileManager1 org.freedesktop.FileManager1.ShowItems array:string:"file:///home/kaan/HW3/13.txt" string:""

        let conn = Connection::session().await.unwrap();
        let proxy = FileManager1Proxy::new(&conn).await.unwrap();

        let uri = format!("file://{}", file_path);
        proxy.show_items(vec![uri], String::new()).await.unwrap();
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("explorer.exe /select,")
            .arg(file_path)
            .output()
            .unwrap();
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open -R").arg(file_path).output().unwrap();
    }
}
