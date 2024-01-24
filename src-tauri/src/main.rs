// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use google_drive::{traits::FileOps, Client};
use tauri::{
    http::ResponseBuilder,
    plugin::{Builder as PluginBuilder, TauriPlugin},
    Manager, Runtime,
};
use url::Url;

fn main() {
    tauri::Builder::default()
        .plugin(build())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Default)]
struct DriveInstance(Mutex<Option<Client>>);

#[tauri::command]
async fn get_file_by_id(
    file_id: String,
    drive_instance: tauri::State<'_, DriveInstance>,
) -> Result<google_drive::types::File, String> {
    let client = drive_instance.0.lock().unwrap().clone().unwrap();
    let result = client
        .files()
        .get(&file_id, false, "published".into(), true, true)
        .await;
    match result {
        Ok(response) => Ok(response.body),
        Err(err) => Err(err.to_string()),
    }
}

fn build<R: Runtime>() -> TauriPlugin<R> {
    PluginBuilder::new("drive")
    .register_uri_scheme_protocol("drive", move |app, req|{
        dbg!(req);
        let url = Url::parse(req.uri()).unwrap();
        let drive_id = url.host_str().unwrap();
        let drive_instance = app.state::<DriveInstance>();
        let client = drive_instance.0.lock().unwrap().clone().unwrap();

        let result = tauri::async_runtime::block_on(async {
            client.files().download_by_id(drive_id).await
        });

        match result {
            Ok(response) => ResponseBuilder::new()
                .header("Content-Type", "application/pdf")
                .status(response.status)
                .body(response.body.to_vec()),
            Err(_) => ResponseBuilder::new()
                .status(500)
                .body("Error".as_bytes().to_vec())
        }
    }) 
    .invoke_handler(tauri::generate_handler![get_file_by_id])
    .setup(|app| {
        tauri::async_runtime::block_on(async move {
            let drive_instance = DriveInstance::default();
            let mut lock = drive_instance.0.lock().unwrap();
            let g_client = Client::new(
                String::from("797001069823-691t69pnl26eo1krgj3b2e1lscjtd255.apps.googleusercontent.com"),
                String::from("VlRrdJl6Vaj4Q5E6yxfwhBZf"),
                String::from("urn:ietf:wg:oauth:2.0:oob"),
                String::from("ya29.a0Adw1xeVElggha9gVbWfZLOyasptYQJP-SFYf06a4F3Qss22OT6W9nj0_9jZGoNG5fRj6kVnIdxW-eshGYCAPcBNY3BAZfIqmR10WNExwj-0PNPOGMTlSxBRHYceMgqUjzRsQVV8IIFD91GjdQIzm6ZRSkoJKFBwcbdkL"),
                String::from("1//0fDlqoAvkr0tXCgYIARAAGA8SNwF-L9Ir5z1lRrgiecJEpEYjmexCWfOC5zqVEcRkvfAhSbcegHprkUkiMNrwEKnzIEmtYrUedW0"),
            );
            g_client.refresh_access_token().await.unwrap();
            *lock = Some(g_client);
            drop(lock);

            app.manage(drive_instance);
        });
        Ok(())
    })
    .build()
}
