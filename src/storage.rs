use std::{fs, io::ErrorKind, path::Path};

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, sql_query};

use crate::error::{AppError, Result};
use crate::note::{Note, NoteStatusFilter};
use crate::schema::notes;
use crate::schema::notes::dsl as notes_dsl;

const DB_FILE: &str = "notes.db";
const LEGACY_JSON_FILE: &str = "notes.json";

fn open_connection() -> Result<SqliteConnection> {
    let mut conn = SqliteConnection::establish(DB_FILE)?;
    run_migrations(&mut conn)?;
    migrate_legacy_json_if_needed(&mut conn)?;
    Ok(conn)
}

fn run_migrations(conn: &mut SqliteConnection) -> Result<()> {
    sql_query(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY
        )",
    )
    .execute(conn)?;

    let current: i64 = diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(
        "COALESCE((SELECT MAX(version) FROM schema_migrations), 0)",
    ))
    .get_result(conn)?;

    if current < 1 {
        sql_query(
            "CREATE TABLE IF NOT EXISTS notes (
                id INTEGER PRIMARY KEY,
                text TEXT NOT NULL,
                done INTEGER NOT NULL CHECK (done IN (0, 1))
            )",
        )
        .execute(conn)?;
        sql_query("INSERT INTO schema_migrations (version) VALUES (1)").execute(conn)?;
    }

    if current < 2 {
        sql_query(
            "ALTER TABLE notes ADD COLUMN created_at TEXT NOT NULL DEFAULT '1970-01-01 00:00:00'",
        )
        .execute(conn)?;
        sql_query(
            "ALTER TABLE notes ADD COLUMN updated_at TEXT NOT NULL DEFAULT '1970-01-01 00:00:00'",
        )
        .execute(conn)?;

        sql_query(
        "UPDATE notes SET created_at = datetime('now') WHERE created_at = '1970-01-01 00:00:00'",
    )
    .execute(conn)?;
        sql_query(
        "UPDATE notes SET updated_at = datetime('now') WHERE updated_at = '1970-01-01 00:00:00'",
    )
    .execute(conn)?;

        sql_query("INSERT INTO schema_migrations (version) VALUES (2)").execute(conn)?;
    }

    if current < 3 {
        sql_query("ALTER TABLE notes ADD COLUMN priority INTEGER NOT NULL DEFAULT (0)")
            .execute(conn)?;
        sql_query("INSERT INTO schema_migrations (version) VALUES (3)").execute(conn)?;
    }

    Ok(())
}

fn migrate_legacy_json_if_needed(conn: &mut SqliteConnection) -> Result<()> {
    let legacy_path = Path::new(LEGACY_JSON_FILE);
    if !legacy_path.exists() {
        return Ok(());
    }

    let count: i64 = notes_dsl::notes.count().get_result(conn)?;

    if count > 0 {
        return Ok(());
    }

    let raw = fs::read_to_string(legacy_path)?;
    let legacy_notes: Vec<Note> = serde_json::from_str(&raw)?;

    conn.transaction::<_, AppError, _>(|tx| {
        for note in &legacy_notes {
            diesel::insert_into(notes::table)
                .values((
                    notes_dsl::id.eq(note.id as i64),
                    notes_dsl::text.eq(&note.text),
                    notes_dsl::done.eq(note.done),
                    notes_dsl::priority.eq(note.priority),
                    notes_dsl::created_at.eq(diesel::dsl::sql::<diesel::sql_types::Timestamp>(
                        "CURRENT_TIMESTAMP",
                    )),
                    notes_dsl::updated_at.eq(diesel::dsl::sql::<diesel::sql_types::Timestamp>(
                        "CURRENT_TIMESTAMP",
                    )),
                ))
                .execute(tx)?;
        }
        Ok(())
    })?;

    let migrated_path = legacy_path.with_extension("json.migrated");
    match fs::rename(legacy_path, &migrated_path) {
        Ok(()) => {}
        Err(e) if e.kind() == ErrorKind::NotFound || e.kind() == ErrorKind::AlreadyExists => {}
        Err(e) => return Err(e.into()),
    }

    Ok(())
}

pub fn add_note(text_input: &str) -> Result<u64> {
    let mut conn = open_connection()?;

    diesel::insert_into(notes::table)
        .values((
            notes_dsl::text.eq(text_input),
            notes_dsl::done.eq(false),
            notes_dsl::priority.eq(0_i64),
            notes_dsl::created_at.eq(diesel::dsl::sql::<diesel::sql_types::Timestamp>(
                "CURRENT_TIMESTAMP",
            )),
            notes_dsl::created_at.eq(diesel::dsl::sql::<diesel::sql_types::Timestamp>(
                "CURRENT_TIMESTAMP",
            )),
        ))
        .execute(&mut conn)?;

    let new_id: i64 = diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(
        "last_insert_rowid()",
    ))
    .get_result(&mut conn)?;

    Ok(new_id as u64)
}

pub fn list_notes(status: NoteStatusFilter, contains: Option<&str>) -> Result<Vec<Note>> {
    let mut conn = open_connection()?;

    let mut query = notes_dsl::notes
        .select((
            notes_dsl::id,
            notes_dsl::text,
            notes_dsl::done,
            notes_dsl::priority,
        ))
        .into_boxed();

    match status {
        NoteStatusFilter::All => {}
        NoteStatusFilter::Done => {
            query = query.filter(notes_dsl::done.eq(true));
        }
        NoteStatusFilter::Todo => {
            query = query.filter(notes_dsl::done.eq(false));
        }
    }

    if let Some(term) = contains {
        let pattern = format!("%{}%", term);
        query = query.filter(notes_dsl::text.like(pattern));
    }

    let rows: Vec<(i64, String, bool, i64)> = query.order(notes_dsl::id.asc()).load(&mut conn)?;

    let notes = rows
        .into_iter()
        .map(|(id, text, done, priority)| Note {
            id: id as u64,
            text,
            done,
            priority,
        })
        .collect();

    Ok(notes)
}

pub fn mark_note_done(target_id: u64) -> Result<()> {
    let mut conn = open_connection()?;
    let db_id = i64::try_from(target_id).map_err(|_| AppError::InvalidId(target_id))?;

    let changed = diesel::update(notes_dsl::notes.filter(notes_dsl::id.eq(db_id)))
        .set((
            notes_dsl::done.eq(true),
            notes_dsl::created_at.eq(diesel::dsl::sql::<diesel::sql_types::Timestamp>(
                "CURRENT_TIMESTAMP",
            )),
        ))
        .execute(&mut conn)?;

    if changed == 0 {
        return Err(AppError::InvalidId(target_id));
    }

    Ok(())
}

pub fn edit_note_text(target_id: u64, new_text: &str) -> Result<()> {
    let mut conn = open_connection()?;
    let db_id = i64::try_from(target_id).map_err(|_| AppError::InvalidId(target_id))?;

    let changed = diesel::update(notes_dsl::notes.filter(notes_dsl::id.eq(db_id)))
        .set((
            notes_dsl::text.eq(new_text),
            notes_dsl::created_at.eq(diesel::dsl::sql::<diesel::sql_types::Timestamp>(
                "CURRENT_TIMESTAMP",
            )),
        ))
        .execute(&mut conn)?;

    if changed == 0 {
        return Err(AppError::InvalidId(target_id));
    }

    Ok(())
}

pub fn remove_note_by_id(target_id: u64) -> Result<()> {
    let mut conn = open_connection()?;
    let db_id = i64::try_from(target_id).map_err(|_| AppError::InvalidId(target_id))?;

    let changed =
        diesel::delete(notes_dsl::notes.filter(notes_dsl::id.eq(db_id))).execute(&mut conn)?;

    if changed == 0 {
        return Err(AppError::InvalidId(target_id));
    }

    Ok(())
}
