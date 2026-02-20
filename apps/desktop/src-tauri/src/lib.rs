// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use novel_core::{
    Note, NoteStatusFilter, Priority, add_note as core_add_note, list_notes as core_list_notes,
};

#[tauri::command]
fn add_note(text: String, priority: i64) -> Result<u64, String> {
    let priority = Priority::try_from(priority).map_err(|e| e.to_string())?;
    core_add_note(&text, priority).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_notes(
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

    core_list_notes(status, contains.as_deref(), priority).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![add_note, list_notes])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
