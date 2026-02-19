mod error;
mod note;
mod storage;

use clap::{Parser, Subcommand};
use error::Result;
use note::{Note, NoteStatusFilter, edit_note, mark_done, next_note_id, remove_note};
use storage::{load_notes, save_notes};

use crate::note::filter_notes;

#[derive(Parser)]
#[command(name = "novel-cli")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add {
        text: String,
    },
    List {
        #[arg(long, conflicts_with = "todo")]
        done: bool,

        #[arg(long, conflicts_with = "done")]
        todo: bool,

        #[arg(long)]
        contains: Option<String>,
    },
    Done {
        id: u64,
    },
    Remove {
        id: u64,
    },
    Edit {
        id: u64,
        text: String,
    },
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
        Commands::Edit { id, text } => {
            edit_note(&mut notes, id, text)?;
            save_notes(&notes)?;
            println!("edited Note #{}", id);
        }
        Commands::List {
            done,
            todo,
            contains,
        } => {
            let status = if done {
                NoteStatusFilter::Done
            } else if todo {
                NoteStatusFilter::Todo
            } else {
                NoteStatusFilter::All
            };

            let filtered_notes = filter_notes(&notes, status, contains.as_deref());

            if filtered_notes.is_empty() {
                println!("No notes found.");
            } else {
                for note in filtered_notes {
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
