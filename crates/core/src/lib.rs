pub mod error;
pub mod note;
pub mod schema;
pub mod storage;

pub use error::{AppError, Result};
pub use note::{Note, NoteStatusFilter, Priority};
pub use storage::{
    DEFAULT_DB_FILE, add_note, add_note_in, edit_note_text, list_notes, list_notes_in,
    mark_note_done, remove_note_by_id, set_note_priority,
};
