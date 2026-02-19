use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Note {
    pub id: u64,
    pub text: String,
    pub done: bool,
}

pub enum NoteStatusFilter {
    All,
    Done,
    Todo,
}
