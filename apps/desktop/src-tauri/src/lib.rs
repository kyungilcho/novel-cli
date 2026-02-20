use novel_core::{
    add_note_in as core_add_note_in, list_notes_in as core_list_notes_in, Note, NoteStatusFilter,
    Priority,
};
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager};

#[tauri::command]
fn add_note(app: AppHandle, text: String, priority: i64) -> Result<u64, String> {
    let db_path = resolve_db_path(&app)?;
    let priority = Priority::try_from(priority).map_err(|e| e.to_string())?;
    core_add_note_in(&db_path, &text, priority).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_notes(
    app: AppHandle,
    done: bool,
    todo: bool,
    contains: Option<String>,
    priority: Option<i64>,
) -> Result<Vec<Note>, String> {
    if done && todo {
        return Err("done and todo cannot be true at the same time".to_string());
    }

    let status = if done {
        NoteStatusFilter::Done
    } else if todo {
        NoteStatusFilter::Todo
    } else {
        NoteStatusFilter::All
    };

    let priority = priority
        .map(Priority::try_from)
        .transpose()
        .map_err(|e| e.to_string())?;

    let db_path = resolve_db_path(&app)?;
    core_list_notes_in(&db_path, status, contains.as_deref(), priority).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![add_note, list_notes])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn resolve_db_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join(novel_core::DEFAULT_DB_FILE))
}
