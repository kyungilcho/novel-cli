use clap::{Parser, Subcommand};
use novel_core::{
    NoteStatusFilter, Priority, Result, add_note, edit_note_text, list_notes, mark_note_done,
    remove_note_by_id, set_note_priority,
};

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
        #[arg(long, default_value_t = 0)]
        priority: i64,
    },
    List {
        #[arg(long, conflicts_with = "todo")]
        done: bool,

        #[arg(long, conflicts_with = "done")]
        todo: bool,

        #[arg(long)]
        contains: Option<String>,

        #[arg(long)]
        priority: Option<i64>,
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
    Priority {
        id: u64,
        priority: i64,
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
        Commands::Add { text, priority } => {
            let priority = Priority::try_from(priority)?;
            let id = add_note(&text, priority)?;

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
            priority,
        } => {
            let status = if done {
                NoteStatusFilter::Done
            } else if todo {
                NoteStatusFilter::Todo
            } else {
                NoteStatusFilter::All
            };

            let priority = match priority {
                Some(value) => Some(Priority::try_from(value)?),
                None => None,
            };

            let filtered_notes = list_notes(status, contains.as_deref(), priority)?;

            if filtered_notes.is_empty() {
                println!("No notes found.");
            } else {
                for note in filtered_notes {
                    let mark = if note.done { "x" } else { " " };
                    println!("[{}] P{}: {} {}", mark, note.priority, note.id, note.text);
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
        Commands::Priority { id, priority } => {
            let priority = Priority::try_from(priority)?;
            set_note_priority(id, priority)?;
            println!("set priority of Note #{} to {}", id, priority.value());
        }
    }

    Ok(())
}
