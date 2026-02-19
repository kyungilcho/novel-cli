use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Note {
    pub id: u64,
    pub text: String,
    pub done: bool,
}

pub fn next_note_id(notes: &[Note]) -> u64 {
    notes.last().map(|n| n.id + 1).unwrap_or(1)
}

pub fn mark_done(notes: &mut [Note], id: u64) -> Result<()> {
    let note = notes
        .iter_mut()
        .find(|n| n.id == id)
        .ok_or(AppError::InvalidId(id))?;

    note.done = true;
    Ok(())
}

pub fn remove_note(notes: &mut Vec<Note>, id: u64) -> Result<()> {
    let idx = notes
        .iter()
        .position(|n| n.id == id)
        .ok_or(AppError::InvalidId(id))?;

    notes.remove(idx);
    Ok(())
}
