use std::fs;
use std::path::Path;
use tauri::{AppHandle, Emitter, Manager};
use crate::utils::db_manager::{create_installation, delete_installation_by_id, get_install_info_by_id, get_installs, get_installs_by_manifest_id, get_manifest_info_by_filename, get_manifest_info_by_id, get_settings, update_install_dxvk_location_by_id, update_install_dxvk_version_by_id, update_install_env_vars_by_id, update_install_fps_value_by_id, update_install_game_location_by_id, update_install_ignore_updates_by_id, update_install_launch_args_by_id, update_install_launch_cmd_by_id, update_install_pre_launch_cmd_by_id, update_install_prefix_location_by_id, update_install_runner_location_by_id, update_install_runner_version_by_id, update_install_skip_hash_check_by_id, update_install_use_fps_unlock_by_id, update_install_use_jadeite_by_id, update_install_use_xxmi_by_id};
use crate::utils::game_launch_manager::launch;
use crate::utils::{copy_dir_all, generate_cuid};
use crate::utils::repo_manager::{get_manifest};

#[tauri::command]
pub async fn list_installs(app: AppHandle) -> Option<String> {
    let installs = get_installs(&app);

    if installs.is_some() {
        let install = installs.unwrap();
        let stringified = serde_json::to_string(&install).unwrap();
        Some(stringified)
    } else {
        None
    }
}

#[tauri::command]
pub fn list_installs_by_manifest_id(app: AppHandle, manifest_id: String) -> Option<String> {
    let installs = get_installs_by_manifest_id(&app, manifest_id);

    if installs.is_some() {
        let install = installs.unwrap();
        let stringified = serde_json::to_string(&install).unwrap();
        Some(stringified)
    } else {
        None
    }
}

#[tauri::command]
pub fn get_install_by_id(app: AppHandle, id: String) -> Option<String> {
    let inst = get_install_info_by_id(&app, id);

    if inst.is_some() {
        let install = inst.unwrap();
        let stringified = serde_json::to_string(&install).unwrap();
        Some(stringified)
    } else {
        None
    }
}

#[tauri::command]
pub async fn add_install(app: AppHandle, manifest_id: String, version: String, name: String, directory: String, mut runner_path: String, mut dxvk_path: String, runner_version: String, dxvk_version: String, game_icon: String, game_background: String, ignore_updates: bool, skip_hash_check: bool, use_jadeite: bool, use_xxmi: bool, use_fps_unlock: bool, env_vars: String, pre_launch_command: String, launch_command: String, fps_value: String, runner_prefix: String, launch_args: String) -> Option<bool> {
    if manifest_id.is_empty() || version.is_empty() || name.is_empty() || directory.is_empty() || runner_path.is_empty() || dxvk_path.is_empty() || game_icon.is_empty() || game_background.is_empty() {
        None
    } else {
        // TODO: Write bullshit to download, unpack, setup the entire installation
        let cuid = generate_cuid();
        let m = manifest_id + ".json";
        let dbm = get_manifest_info_by_filename(&app, m.clone()).unwrap();
        let gm = get_manifest(&app, m.clone()).unwrap();
        let g = gm.game_versions.iter().find(|e| e.metadata.version == version).unwrap();

        let data_path = app.path().app_data_dir().unwrap();
        let comppath = data_path.join("compatibility");
        let wine = comppath.join("runners");
        let dxvk = comppath.join("dxvk");
        let prefixes = comppath.join("prefixes");

        #[cfg(target_os = "linux")]
        {
            if !comppath.exists() {
                fs::create_dir_all(&wine).unwrap();
                fs::create_dir_all(&dxvk).unwrap();
                fs::create_dir_all(&prefixes).unwrap();
            }
            runner_path = wine.join(runner_version.clone()).to_str().unwrap().to_string();
            dxvk_path = dxvk.join(dxvk_version.clone()).to_str().unwrap().to_string();

            if !Path::exists(runner_path.as_ref()) {
                fs::create_dir_all(runner_path.clone()).unwrap();
            }

            if !Path::exists(dxvk_path.as_ref()) {
                fs::create_dir_all(dxvk_path.clone()).unwrap();
            }

            if !Path::exists(runner_prefix.as_ref()) {
                fs::create_dir_all(runner_prefix.clone()).unwrap();
            }
        }
        create_installation(&app, cuid, dbm.id, version, g.metadata.versioned_name.clone(), directory, runner_path, dxvk_path, runner_version, dxvk_version, g.assets.game_icon.clone(), g.assets.game_background.clone(), ignore_updates, skip_hash_check, use_jadeite, use_xxmi, use_fps_unlock, env_vars, pre_launch_command, launch_command, fps_value, runner_prefix, launch_args).unwrap();
        Some(true)
    }
}

#[tauri::command]
pub async fn remove_install(app: AppHandle, id: String) -> Option<bool> {
    if id.is_empty() {
        None
    } else {
        // TODO: Write more bullshit to uninstall the installation and wipe its files
        delete_installation_by_id(&app, id).unwrap();
        Some(true)
    }
}

#[tauri::command]
pub fn update_install_game_path(app: AppHandle, id: String, path: String) -> Option<bool> {
    let install = get_install_info_by_id(&app, id);

    if install.is_some() {
        let m = install.unwrap();
        let np = path.clone();
        let app1 = app.clone();
        if !Path::exists(path.as_ref()) {
            fs::create_dir_all(path.clone()).unwrap();
        }

        std::thread::spawn(move || {
            let oldpath = Path::new(m.directory.as_str());
            copy_dir_all(&app1, oldpath.to_owned(), &path.to_owned().clone()).unwrap();
            app1.emit("move_complete", &path).unwrap();
        });
        update_install_game_location_by_id(&app, m.id, np);
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_install_runner_path(app: AppHandle, id: String, path: String) -> Option<bool> {
    let install = get_install_info_by_id(&app, id);

    if install.is_some() {
        let m = install.unwrap();
        let np = path.clone();
        let app1 = app.clone();
        if !Path::exists(path.as_ref()) {
            fs::create_dir_all(path.clone()).unwrap();
        }

        std::thread::spawn(move || {
            let oldpath = Path::new(m.runner_path.as_str());
            copy_dir_all(&app1, oldpath.to_owned(), &path.to_owned().clone()).unwrap();
            app1.emit("move_complete", &path).unwrap();
        });
        update_install_runner_location_by_id(&app, m.id, np);
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_install_dxvk_path(app: AppHandle, id: String, path: String) -> Option<bool> {
    let install = get_install_info_by_id(&app, id);

    if install.is_some() {
        let m = install.unwrap();
        let np = path.clone();
        let app1 = app.clone();
        if !Path::exists(path.as_ref()) {
            fs::create_dir_all(path.clone()).unwrap();
        }

        std::thread::spawn(move || {
            let oldpath = Path::new(m.dxvk_path.as_str());
            copy_dir_all(&app1, oldpath.to_owned(), &path.to_owned().clone()).unwrap();
            app1.emit("move_complete", &path).unwrap();
        });
        update_install_dxvk_location_by_id(&app, m.id, np);
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_install_skip_version_updates(app: AppHandle, id: String, enabled: bool) -> Option<bool> {
    let manifest = get_install_info_by_id(&app, id);

    if manifest.is_some() {
        let m = manifest.unwrap();
        update_install_ignore_updates_by_id(&app, m.id, enabled);
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_install_skip_hash_valid(app: AppHandle, id: String, enabled: bool) -> Option<bool> {
    let manifest = get_install_info_by_id(&app, id);

    if manifest.is_some() {
        let m = manifest.unwrap();
        update_install_skip_hash_check_by_id(&app, m.id, enabled);
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_install_use_jadeite(app: AppHandle, id: String, enabled: bool) -> Option<bool> {
    let manifest = get_install_info_by_id(&app, id);

    if manifest.is_some() {
        let m = manifest.unwrap();
        update_install_use_jadeite_by_id(&app, m.id, enabled);
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_install_use_xxmi(app: AppHandle, id: String, enabled: bool) -> Option<bool> {
    let manifest = get_install_info_by_id(&app, id);

    if manifest.is_some() {
        let m = manifest.unwrap();
        update_install_use_xxmi_by_id(&app, m.id, enabled);
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_install_use_fps_unlock(app: AppHandle, id: String, enabled: bool) -> Option<bool> {
    let manifest = get_install_info_by_id(&app, id);

    if manifest.is_some() {
        let m = manifest.unwrap();
        update_install_use_fps_unlock_by_id(&app, m.id, enabled);
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_install_fps_value(app: AppHandle, id: String, fps: String) -> Option<bool> {
    let install = get_install_info_by_id(&app, id);

    if install.is_some() {
        let m = install.unwrap();
        update_install_fps_value_by_id(&app, m.id, fps);
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_install_env_vars(app: AppHandle, id: String, env_vars: String) -> Option<bool> {
    let install = get_install_info_by_id(&app, id);

    if install.is_some() {
        let m = install.unwrap();
        update_install_env_vars_by_id(&app, m.id, env_vars);
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_install_pre_launch_cmd(app: AppHandle, id: String, cmd: String) -> Option<bool> {
    let install = get_install_info_by_id(&app, id);

    if install.is_some() {
        let m = install.unwrap();
        update_install_pre_launch_cmd_by_id(&app, m.id, cmd);
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_install_launch_cmd(app: AppHandle, id: String, cmd: String) -> Option<bool> {
    let install = get_install_info_by_id(&app, id);

    if install.is_some() {
        let m = install.unwrap();
        update_install_launch_cmd_by_id(&app, m.id, cmd);
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_install_prefix_path(app: AppHandle, id: String, path: String) -> Option<bool> {
    let install = get_install_info_by_id(&app, id);

    if install.is_some() {
        let m = install.unwrap();
        let np = path.clone();
        let app1 = app.clone();
        if !Path::exists(path.as_ref()) {
            fs::create_dir_all(path.clone()).unwrap();
        }

        std::thread::spawn(move || {
            let oldpath = Path::new(m.runner_prefix.as_str());
            copy_dir_all(&app1, oldpath.to_owned(), &path.to_owned().clone()).unwrap();
            app1.emit("move_complete", &path).unwrap();
        });
        update_install_prefix_location_by_id(&app, m.id, np);
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_install_launch_args(app: AppHandle, id: String, args: String) -> Option<bool> {
    let install = get_install_info_by_id(&app, id);

    if install.is_some() {
        let m = install.unwrap();
        update_install_launch_args_by_id(&app, m.id, args);
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_install_runner_version(app: AppHandle, id: String, version: String) -> Option<bool> {
    let install = get_install_info_by_id(&app, id);

    if install.is_some() {
        let m = install.unwrap();
        // TODO: Download runner version
        let rp = m.runner_path.clone();
        let rpn = rp.replace(m.runner_version.as_str(), version.as_str());
        if !Path::exists(rpn.as_ref()) {
            fs::create_dir_all(rpn.clone()).unwrap();
        }

        update_install_runner_version_by_id(&app, m.id.clone(), version);
        update_install_runner_location_by_id(&app, m.id, rpn);
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_install_dxvk_version(app: AppHandle, id: String, version: String) -> Option<bool> {
    let install = get_install_info_by_id(&app, id);

    if install.is_some() {
        let m = install.unwrap();
        // TODO: Download DXVK version
        let p = m.dxvk_path.clone();
        let pn = p.replace(m.dxvk_version.as_str(), version.as_str());
        if !Path::exists(pn.as_ref()) {
            fs::create_dir_all(pn.clone()).unwrap();
        }

        update_install_dxvk_version_by_id(&app, m.id.clone(), version);
        update_install_dxvk_location_by_id(&app, m.id, pn);
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn game_launch(app: AppHandle, id: String) -> Option<bool> {
    let install = get_install_info_by_id(&app, id);
    let global_settings = get_settings(&app).unwrap();

    if install.is_some() {
        let m = install.unwrap();
        let gmm = get_manifest_info_by_id(&app, m.clone().manifest_id).unwrap();
        let gm = get_manifest(&app, gmm.filename).unwrap();

        let rslt = launch(&app, m.clone(), gm, global_settings);
        if rslt.is_ok() {
            Some(true)
        } else {
            None
        }
    } else {
        None
    }
}