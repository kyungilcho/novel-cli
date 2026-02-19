use std::{fs, io::ErrorKind, path::Path};

use crate::error::Result;
use crate::note::Note;

const NOTES_FILE: &str = "notes.json";

pub fn load_notes() -> Result<Vec<Note>> {
    if !Path::new(NOTES_FILE).exists() {
        return Ok(Vec::new());
    }

    let raw = fs::read_to_string(NOTES_FILE)?;
    let notes = serde_json::from_str(&raw)?;
    Ok(notes)
}

pub fn save_notes(notes: &[Note]) -> Result<()> {
    let raw = serde_json::to_string_pretty(notes)?;
    let notes_path = Path::new(NOTES_FILE);
    let tmp_path = notes_path.with_extension("json.tmp");

    match fs::remove_file(&tmp_path) {
        Ok(()) => {}
        Err(e) if e.kind() == ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }

    fs::write(&tmp_path, raw)?;
    fs::rename(&tmp_path, notes_path)?;
    Ok(())
}
