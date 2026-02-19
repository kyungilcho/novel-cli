pub mod error;
pub mod note;
pub mod schema;
pub mod storage;

pub use error::{AppError, Result};
pub use note::{Note, NoteStatusFilter, Priority};
pub use storage::{
    add_note, edit_note_text, list_notes, mark_note_done, remove_note_by_id, set_note_priority,
};
