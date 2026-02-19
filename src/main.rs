mod error;
mod note;
mod schema;
mod storage;

use clap::{Parser, Subcommand};
use error::Result;
use note::NoteStatusFilter;
use storage::{add_note, edit_note_text, list_notes, mark_note_done, remove_note_by_id};

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

    match cli.command {
        Commands::Add { text } => {
            let id = add_note(&text)?;

            println!("adding Note #{}", id);
        }
        Commands::Edit { id, text } => {
            edit_note_text(id, &text)?;
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

            let filtered_notes = list_notes(status, contains.as_deref())?;

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
            mark_note_done(id)?;
            println!("marked Note #{} as done", id);
        }
        Commands::Remove { id } => {
            remove_note_by_id(id)?;
            println!("removed Note #{}", id);
        }
    }

    Ok(())
}
