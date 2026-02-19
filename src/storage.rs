use std::{fs, io::ErrorKind, path::Path};

use rusqlite::{Connection, params};

use crate::error::{AppError, Result};
use crate::note::{Note, NoteStatusFilter};

// SQLite file used by the current storage backend.
const DB_FILE: &str = "notes.db";

// Previous JSON file used by the legacy storage backend.
// We migrate from this file once and then archive it.
const LEGACY_JSON_FILE: &str = "notes.json";

/// Open the database connection and ensure storage is ready.
///
/// Design intent:
/// - Callers (`load_notes`, `save_notes`) should not need to remember
///   schema creation or legacy migration details.
/// - Centralizing these bootstrapping steps here keeps call sites simple.
fn open_connection() -> Result<Connection> {
    // `Connection::open` creates the DB file if it does not exist.
    // `?` propagates errors and converts them into our AppError via `From`.
    let mut conn = Connection::open(DB_FILE)?;

    // run migration
    run_migrations(&conn)?;

    // If needed, migrate data from the legacy JSON file into SQLite.
    migrate_legacy_json_if_needed(&mut conn)?;

    Ok(conn)
}

fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY
        )",
        [],
    )?;

    let current: i64 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )?;

    if current < 1 {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS notes (
                id INTEGER PRIMARY KEY,
                text TEXT NOT NULL,
                done INTEGER NOT NULL CHECK (done IN (0, 1))
            )",
            [],
        )?;

        conn.execute("INSERT INTO schema_migrations (version) VALUES (1)", [])?;
    }

    if current < 2 {
        conn.execute(
            "ALTER TABLE notes ADD COLUMN created_at TEXT NOT NULL DEFAULT (datetime('now'))",
            [],
        )?;
        conn.execute(
            "ALTER TABLE notes ADD COLUMN updated_at TEXT NOT NULL DEFAULT (datetime('now'))",
            [],
        )?;

        conn.execute("INSERT INTO schema_migrations (version) VALUES (2)", [])?;
    }

    Ok(())
}

/// One-time migration from `notes.json` to SQLite.
///
/// Rules:
/// 1) If the legacy file does not exist, do nothing.
/// 2) If DB already has rows, assume migration already happened.
/// 3) Insert inside a transaction so migration is atomic.
fn migrate_legacy_json_if_needed(conn: &mut Connection) -> Result<()> {
    let legacy_path = Path::new(LEGACY_JSON_FILE);

    // Early return keeps no-op path explicit and cheap.
    if !legacy_path.exists() {
        return Ok(());
    }

    // `query_row` executes one-row SQL and maps that row via closure.
    // `row.get(0)` reads the first selected column (`COUNT(*)`).
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))?;

    // If table is already populated, skip migration to avoid duplicates.
    if count > 0 {
        return Ok(());
    }

    // Read legacy JSON and deserialize into typed notes.
    let raw = fs::read_to_string(legacy_path)?;
    let notes: Vec<Note> = serde_json::from_str(&raw)?;

    // Transaction: either all rows are inserted, or none are.
    let tx = conn.transaction()?;

    // This extra block is intentional.
    // `stmt` borrows `tx`; ending the block drops `stmt` before `commit`.
    {
        // SQL placeholders use positional binding (?1, ?2, ?3).
        let mut stmt = tx.prepare("INSERT INTO notes (id, text, done) VALUES (?1, ?2, ?3)")?;

        // Consuming `notes` is fine here because vector is no longer needed.
        for note in notes {
            // `params![]` binds values to SQL placeholders in order.
            stmt.execute(params![note.id, note.text, note.done])?;
        }
    }

    tx.commit()?;

    // Keep an archival marker instead of deleting permanently.
    // `with_extension("json.migrated")` turns `notes.json` into
    // `notes.json.migrated`.
    let migrated_path = legacy_path.with_extension("json.migrated");
    match fs::rename(legacy_path, &migrated_path) {
        Ok(()) => {}
        // NotFound: file already moved/removed.
        // AlreadyExists: archive file already exists.
        Err(e) if e.kind() == ErrorKind::NotFound || e.kind() == ErrorKind::AlreadyExists => {}
        Err(e) => return Err(e.into()),
    }

    Ok(())
}

pub fn add_note(text: &str) -> Result<u64> {
    let conn = open_connection()?;
    conn.execute(
        "INSERT INTO notes (text, done, created_at, updated_at) VALUES (?1, 0, datetime('now'), datetime('now'))",
        params![text],
    )?;
    Ok(conn.last_insert_rowid() as u64)
}

pub fn list_notes(status: NoteStatusFilter, contains: Option<&str>) -> Result<Vec<Note>> {
    let conn = open_connection()?;

    let sql = match status {
        NoteStatusFilter::All => {
            "SELECT id, text, done FROM notes
             WHERE (?1 IS NULL OR instr(text, ?1) > 0)
             ORDER BY id"
        }
        NoteStatusFilter::Done => {
            "SELECT id, text, done FROM notes
             WHERE done = 1 AND (?1 IS NULL OR instr(text, ?1) > 0)
             ORDER BY id"
        }
        NoteStatusFilter::Todo => {
            "SELECT id, text, done FROM notes
             WHERE done = 0 AND (?1 IS NULL OR instr(text, ?1) > 0)
             ORDER BY id"
        }
    };

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(params![contains], |row| {
        Ok(Note {
            id: row.get(0)?,
            text: row.get(1)?,
            done: row.get(2)?,
        })
    })?;

    let notes = rows.collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(notes)
}

pub fn mark_note_done(id: u64) -> Result<()> {
    let conn = open_connection()?;

    let changed = conn.execute("UPDATE notes SET done = 1 WHERE id = ?1", params![id])?;

    if changed == 0 {
        return Err(AppError::InvalidId(id));
    }

    Ok(())
}

pub fn edit_note_text(id: u64, text: &str) -> Result<()> {
    let conn = open_connection()?;

    let changed = conn.execute(
        "UPDATE notes SET text = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![text, id],
    )?;

    if changed == 0 {
        return Err(AppError::InvalidId(id));
    }

    Ok(())
}

pub fn remove_note_by_id(id: u64) -> Result<()> {
    let conn = open_connection()?;

    let changed = conn.execute("DELETE FROM notes WHERE id = ?1", params![id])?;

    if changed == 0 {
        return Err(AppError::InvalidId(id));
    }

    Ok(())
}
