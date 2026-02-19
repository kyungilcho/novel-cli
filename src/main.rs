mod error;
mod note;
mod storage;

use clap::{Parser, Subcommand};
use error::Result;
use note::{Note, mark_done, next_note_id, remove_note};
use storage::{load_notes, save_notes};

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

fn main() {
    if let Err(e) = run() {
        eprint!("error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    let mut notes = load_notes()?;

    match cli.command {
        Commands::Add { text } => {
            let next_id = next_note_id(&notes);

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
            mark_done(&mut notes, id)?;
            save_notes(&notes)?;
            println!("marked Note #{} as done", id);
        }
        Commands::Remove { id } => {
            remove_note(&mut notes, id)?;
            save_notes(&notes)?;
            println!("removed Note #{}", id);
        }
    }

    Ok(())
}
