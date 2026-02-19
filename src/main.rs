mod error;

use clap::{Parser, Subcommand};
use error::{AppError, Result};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(name = "novel-cli")]

struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add { text: String },
    List,
    Done { id: u64 },
    Remove { id: u64 },
}

#[derive(Debug, Serialize, Deserialize)]
struct Note {
    id: u64,
    text: String,
    done: bool,
}

fn main() {
    if let Err(e) = run() {
        eprint!("error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    let mut notes: Vec<Note> = load_notes()?;

    match cli.command {
        Commands::Add { text } => {
            let next_id = notes.last().map(|n| n.id + 1).unwrap_or(1);

            notes.push(Note {
                id: next_id,
                text,
                done: false,
            });

            save_notes(&notes)?;

            println!("adding Note #{}", next_id);
        }
        Commands::List => {
            if notes.is_empty() {
                println!("No notes found.");
            } else {
                for note in notes {
                    let mark = if note.done { "x" } else { " " };
                    println!("[{}] {}: {}", mark, note.id, note.text);
                }
            }
        }
        Commands::Done { id } => {
            let note = notes
                .iter_mut()
                .find(|n| n.id == id)
                .ok_or(AppError::InvalidId(id))?;

            note.done = true;
            save_notes(&notes)?;
            println!("marked Note #{} as done", id);
        }
        Commands::Remove { id } => {
            let idx = notes
                .iter()
                .position(|n| n.id == id)
                .ok_or(AppError::InvalidId(id))?;

            notes.remove(idx);
            save_notes(&notes)?;
            println!("removed Note #{}", id);
        }
    }

    Ok(())
}

use std::{fs, path::Path};

const NOTES_FILE: &str = "notes.json";

fn load_notes() -> Result<Vec<Note>> {
    if !Path::new(NOTES_FILE).exists() {
        return Ok(Vec::new());
    }

    let raw = fs::read_to_string(NOTES_FILE)?;
    let notes = serde_json::from_str(&raw)?;
    Ok(notes)
}

fn save_notes(notes: &[Note]) -> Result<()> {
    let raw = serde_json::to_string_pretty(notes)?;
    fs::write(NOTES_FILE, raw)?;
    Ok(())
}
