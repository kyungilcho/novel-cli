use std::{fs, path::Path};

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
    fs::write(NOTES_FILE, raw)?;
    Ok(())
}
