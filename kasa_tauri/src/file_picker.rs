use ashpd::desktop::file_chooser::OpenFileRequest;

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
