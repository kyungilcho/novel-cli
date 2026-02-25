use std::path::Path;

#[tauri::command]
pub fn init_repo(root: String) -> Result<(), String> {
    workspace_core::init_repo(Path::new(&root)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn commit(root: String, message: String) -> Result<String, String> {
    workspace_core::commit(Path::new(&root), &message).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn log(root: String) -> Result<Vec<workspace_core::VersionNode>, String> {
    workspace_core::log(Path::new(&root)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn repo_state(root: String) -> Result<workspace_core::RepoState, String> {
    workspace_core::repo_state(Path::new(&root)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn checkout(root: String, node_id: String) -> Result<(), String> {
    workspace_core::checkout(Path::new(&root), &node_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn diff_nodes(
    root: String,
    from: String,
    to: String,
) -> Result<workspace_core::NodeDiff, String> {
    workspace_core::diff_nodes(Path::new(&root), &from, &to).map_err(|e| e.to_string())
}
