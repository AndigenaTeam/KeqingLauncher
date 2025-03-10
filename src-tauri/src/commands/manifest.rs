use linked_hash_map::LinkedHashMap;
use tauri::{AppHandle};
use crate::utils::db_manager::{get_manifest_info_by_filename, get_manifest_info_by_id, get_manifests_by_repository_id, update_manifest_enabled_by_id};
use crate::utils::repo_manager::{get_manifest, get_manifests, GameManifest};

#[tauri::command]
pub fn get_manifest_by_id(app: AppHandle, id: String) -> Option<String> {
    let manifest = get_manifest_info_by_id(&app, id);

    if manifest.is_some() {
        let m = manifest.unwrap();
        let stringified = serde_json::to_string(&m).unwrap();
        Some(stringified)
    } else {
        None
    }
}

#[tauri::command]
pub fn get_manifest_by_filename(app: AppHandle, filename: String) -> Option<String> {
    let manifest = get_manifest_info_by_filename(&app, filename);

    if manifest.is_some() {
        let m = manifest.unwrap();
        let stringified = serde_json::to_string(&m).unwrap();
        Some(stringified)
    } else {
        None
    }
}

#[tauri::command]
pub fn list_manifests_by_repository_id(app: AppHandle, repository_id: String) -> Option<String> {
    let manifests = get_manifests_by_repository_id(&app, repository_id);

    if manifests.is_some() {
        let manifest = manifests.unwrap();
        let stringified = serde_json::to_string(&manifest).unwrap();
        Some(stringified)
    } else {
        None
    }
}

#[tauri::command]
pub fn list_game_manifests(app: AppHandle) -> Option<String> {
    let manifestss: LinkedHashMap<String, GameManifest> = get_manifests(&app);
    let mut manifests: Vec<GameManifest> = Vec::new();

    for value in manifestss.into_iter().map(|(_, value)| value) {
        manifests.push(value);
    }
    
    if manifests.is_empty() {
        None
    } else {
        let stringified = serde_json::to_string(&manifests).unwrap();
        Some(stringified)
    }
}

#[tauri::command]
pub fn get_game_manifest_by_filename(app: AppHandle, filename: String) -> Option<String> {
    let manifest = get_manifest(&app, &filename);
    let db_manifest = get_manifest_info_by_filename(&app, filename.clone());

    if manifest.is_some() && db_manifest.is_some() {
        let dbm = db_manifest.unwrap();

        if dbm.enabled {
            let m = manifest.unwrap();
            let stringified = serde_json::to_string(&m).unwrap();

            Some(stringified)
        } else {
            None
        }
    } else {
        None
    }
}

#[tauri::command]
pub fn update_manifest_enabled(app: AppHandle, id: String, enabled: bool) -> Option<bool> {
    let manifest = get_manifest_info_by_id(&app, id);

    if manifest.is_some() {
        let m = manifest.unwrap();
        update_manifest_enabled_by_id(&app, m.id, enabled);
        Some(true)
    } else {
        None
    }
}