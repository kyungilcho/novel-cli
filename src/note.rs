use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Note {
    pub id: u64,
    pub text: String,
    pub done: bool,
    #[serde(default)]
    pub priority: i64,
}

pub enum NoteStatusFilter {
    All,
    Done,
    Todo,
}
