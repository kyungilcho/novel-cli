use std::{fs, path::Path};

use tauri::{AppHandle, Manager};

#[tauri::command]
pub fn open_project(root: String) -> Result<workspace_core::ProjectInfo, String> {
    workspace_core::open_project(Path::new(&root)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn default_workspace_root(app: AppHandle) -> Result<String, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("workspace");

    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    Ok(dir.to_string_lossy().into_owned())
}
