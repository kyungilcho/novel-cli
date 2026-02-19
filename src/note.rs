use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Note {
    pub id: u64,
    pub text: String,
    pub done: bool,
}

pub fn edit_note(notes: &mut [Note], id: u64, text: String) -> Result<()> {
    let note = notes
        .iter_mut()
        .find(|n| n.id == id)
        .ok_or(AppError::InvalidId(id))?;

    note.text = text;
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppError;

    fn sample_notes() -> Vec<Note> {
        vec![
            Note {
                id: 1,
                text: "first".to_string(),
                done: false,
            },
            Note {
                id: 2,
                text: "second".to_string(),
                done: false,
            },
        ]
    }

    // The first note ID should start at 1 when the list is emtpy
    #[test]
    fn next_note_id_empty_starts_at_one() {
        let notes = Vec::new();
        assert_eq!(next_note_id(&notes), 1);
    }

    // The next note ID should be the last ID + 1
    #[test]
    fn next_note_id_increments_last_id() {
        let notes = sample_notes();
        assert_eq!(next_note_id(&notes), 3);
    }

    // Marking a valid ID as done should set its done field to true.
    #[test]
    fn mark_done_success() {
        let mut notes = sample_notes();
        mark_done(&mut notes, 1).unwrap();
        assert!(notes[0].done);
    }

    // Marking a missing ID as done should return InvalidId error.
    #[test]
    fn mark_done_invalid_id() {
        let mut notes = sample_notes();
        let result = mark_done(&mut notes, 999);
        assert!(matches!(result, Err(AppError::InvalidId(999))));
    }

    // Editing a valid ID should update that note's text.
    #[test]
    fn edit_note_success() {
        let mut notes = sample_notes();
        edit_note(&mut notes, 2, "updated".to_string()).unwrap();
        assert_eq!(notes[1].text, "updated");
    }

    // Editing a missing ID should return InvalidId error.
    #[test]
    fn edit_note_invalid_id() {
        let mut notes = sample_notes();
        let error = edit_note(&mut notes, 99, "xx".to_string()).unwrap_err();
        assert!(matches!(error, AppError::InvalidId(99)));
    }
}
