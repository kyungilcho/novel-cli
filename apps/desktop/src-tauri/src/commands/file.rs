use std::path::Path;

#[tauri::command]
pub fn list_files(root: String, rel: String) -> Result<Vec<workspace_core::FileEntry>, String> {
    workspace_core::list_files(Path::new(&root), &rel).map_err(|e| format!("{:?}", e))
}

#[tauri::command]
pub fn read_file(root: String, rel: String) -> Result<String, String> {
    workspace_core::read_file(Path::new(&root), &rel).map_err(|e| format!("{:?}", e))
}

#[tauri::command]
pub fn write_file(root: String, rel: String, content: String) -> Result<(), String> {
    workspace_core::write_file(Path::new(&root), &rel, &content).map_err(|e| format!("{:?}", e))
}

#[tauri::command]
pub fn create_file(root: String, rel: String, name: String) -> Result<String, String> {
    let rel_path =
        workspace_core::create_file(Path::new(&root), &rel, &name).map_err(|e| e.to_string())?;
    Ok(rel_path.to_string_lossy().into_owned())
}
