use ashpd::desktop::file_chooser::{FileFilter, OpenFileRequest, SaveFileRequest};
use serde::{Deserialize, Serialize};
use zbus::Connection;
use zbus_macros::proxy;
#[tauri::command]
#[specta::specta]
pub async fn new_linux_file_picker_dialog_multiple_folder_select() -> Vec<String> {
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
            .iter()
            .map(|uri| uri.path().to_string())
            .collect()
    }

    #[cfg(not(target_os = "linux"))]
    vec![]
}

pub async fn new_linux_file_picker_dialog_single_folder_select() -> String {
    #[cfg(target_os = "linux")]
    {
        let response = OpenFileRequest::default()
            .modal(true)
            .accept_label("Select")
            .multiple(false)
            .directory(true)
            .send()
            .await
            .unwrap()
            .response()
            .unwrap();

        response
            .uris()
            .first()
            .map(|uri| uri.path().to_string())
            .unwrap_or_default()
    }

    #[cfg(not(target_os = "linux"))]
    String::new()
}

#[tauri::command]
#[specta::specta]
pub async fn new_linux_file_picker_dialog_save_file(
    filter_name: String,
    filter_glob: String,
    current_name: String,
) -> Vec<String> {
    #[cfg(target_os = "linux")]
    {
        let response = SaveFileRequest::default()
            .modal(true)
            .current_name(&*current_name)
            .filters([FileFilter::new(&filter_name).glob(&filter_glob)])
            .send()
            .await
            .unwrap()
            .response()
            .unwrap();

        response
            .uris()
            .iter()
            .map(|uri| uri.path().to_string())
            .collect()
    }
}

#[tauri::command]
#[specta::specta]
pub async fn new_linux_file_picker_dialog_file_select(
    filter_name: String,
    filter_glob: String,
) -> Vec<String> {
    #[cfg(target_os = "linux")]
    {
        let response = OpenFileRequest::default()
            .modal(true)
            .accept_label("Select")
            .multiple(false)
            .filters([FileFilter::new(&filter_name).glob(&filter_glob)])
            .send()
            .await
            .unwrap()
            .response()
            .unwrap();

        response
            .uris()
            .iter()
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
