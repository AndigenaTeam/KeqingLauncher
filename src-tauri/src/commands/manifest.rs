use tauri::{AppHandle};
use crate::utils::db_manager::{get_manifest_info_by_filename, get_manifest_info_by_id, get_manifests_by_repository_id};

#[tauri::command]
pub async fn get_manifest_by_id(app: AppHandle, id: String) -> Option<String> {
    let manifest = get_manifest_info_by_id(&app, id).await;

    if manifest.is_some() {
        let m = manifest.unwrap();
        let stringified = serde_json::to_string(&m).unwrap();
        Some(stringified)
    } else {
        None
    }

}

#[tauri::command]
pub async fn get_manifest_by_filename(app: AppHandle, filename: String) -> Option<String> {
    let manifest = get_manifest_info_by_filename(&app, filename).await;

    if manifest.is_some() {
        let m = manifest.unwrap();
        let stringified = serde_json::to_string(&m).unwrap();
        Some(stringified)
    } else {
        None
    }
}

#[tauri::command]
pub async fn list_manifests_by_repository_id(app: AppHandle, repository_id: String) -> Option<String> {
    let manifests = get_manifests_by_repository_id(&app, repository_id).await;

    if manifests.is_some() {
        let manifest = manifests.unwrap();
        let stringified = serde_json::to_string(&manifest).unwrap();
        Some(stringified)
    } else {
        None
    }
}