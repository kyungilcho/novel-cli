use std::path::Path;

#[tauri::command]
pub fn open_project(root: String) -> Result<workspace_core::ProjectInfo, String> {
    workspace_core::open_project(Path::new(&root)).map_err(|e| format!("{:?}", e))
}
