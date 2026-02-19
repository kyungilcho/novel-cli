use std::{fs, io::ErrorKind, path::Path};

use rusqlite::{Connection, params};

use crate::error::Result;
use crate::note::Note;

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

    // Always safe to call; SQL uses IF NOT EXISTS.
    ensure_schema(&conn)?;

    // If needed, migrate data from the legacy JSON file into SQLite.
    migrate_legacy_json_if_needed(&mut conn)?;

    Ok(conn)
}

/// Ensure the `notes` table exists.
///
/// SQL details:
/// - `CREATE TABLE IF NOT EXISTS` is idempotent.
/// - `id INTEGER PRIMARY KEY` defines the primary key column.
/// - `done` is stored as INTEGER because SQLite has no strict BOOLEAN type.
/// - `CHECK (done IN (0, 1))` enforces boolean-like values.
fn ensure_schema(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY,
            text TEXT NOT NULL,
            done INTEGER NOT NULL CHECK (done IN (0, 1))
        )",
        // No SQL bind parameters for this statement.
        [],
    )?;
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

/// Load all notes from SQLite ordered by id.
///
/// Rust/SQL flow:
/// - `prepare` compiles SQL statement.
/// - `query_map` iterates rows and maps each row into `Note`.
/// - `collect::<Result<Vec<_>, _>>()` folds iterator of row results into one
///   result value (`Ok(Vec<Note>)` or first encountered error).
pub fn load_notes() -> Result<Vec<Note>> {
    let conn = open_connection()?;

    let mut stmt = conn.prepare("SELECT id, text, done FROM notes ORDER BY id")?;

    let rows = stmt.query_map([], |row| {
        Ok(Note {
            // `row.get(index)` reads column by selected position.
            id: row.get(0)?,
            text: row.get(1)?,
            done: row.get(2)?,
        })
    })?;

    // Turbofish here specifies the concrete collection target type.
    let notes = rows.collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(notes)
}

/// Persist current in-memory notes snapshot into SQLite.
///
/// Current strategy is "replace-all":
/// 1) begin transaction
/// 2) delete existing rows
/// 3) reinsert current snapshot
/// 4) commit
///
/// This is simple and deterministic for a local CLI app.
pub fn save_notes(notes: &[Note]) -> Result<()> {
    let mut conn = open_connection()?;
    let tx = conn.transaction()?;

    tx.execute("DELETE FROM notes", [])?;

    // Same scoping reason as migration: drop statement before commit.
    {
        let mut stmt = tx.prepare("INSERT INTO notes (id, text, done) VALUES (?1, ?2, ?3)")?;

        // `notes` is `&[Note]`, so this loop yields `&Note` items.
        for note in notes {
            stmt.execute(params![note.id, note.text, note.done])?;
        }
    }

    tx.commit()?;

    Ok(())
}
