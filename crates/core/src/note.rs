use crate::error::AppError;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Priority(i64);

impl Priority {
    pub fn value(self) -> i64 {
        self.0
    }
}

impl TryFrom<i64> for Priority {
    type Error = AppError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        if !(0..=5).contains(&value) {
            return Err(AppError::InvalidPriority(value));
        }
        Ok(Self(value))
    }
}
