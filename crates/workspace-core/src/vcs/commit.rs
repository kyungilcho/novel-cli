use std::fs;
use std::path::Path;

use diesel::prelude::*;
use diesel_migrations::MigrationHarness;

use crate::vcs::db::{MIGRATIONS, open_connection, to_io};
use crate::vcs::snapshot::{
    SnapshotFile, blob_id_for_content, collect_files_in_workspace, normalize_rel_path,
};
use crate::{NodeId, Result, WorkSpaceError};

pub fn commit(root: &Path, message: &str) -> Result<NodeId> {
    use crate::schema::head::dsl as head_dsl;
    use crate::schema::node_parents::dsl as node_parents_dsl;
    use crate::schema::nodes::dsl as nodes_dsl;

    let message_text = message.trim();

    if message_text.is_empty() {
        return Err(WorkSpaceError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "empty commit message",
        )));
    }

    let mut conn = open_connection(root)?;
    conn.run_pending_migrations(MIGRATIONS).map_err(to_io)?;

    let files = collect_files_in_workspace(root)?;
    let mut snapshot_files = Vec::with_capacity(files.len());

    for rel in files {
        let abs = root.join(&rel);

        let content = fs::read(&abs).map_err(to_io)?;

        let blob_id = blob_id_for_content(&content);

        snapshot_files.push(SnapshotFile {
            path: normalize_rel_path(&rel),
            blob_id,
            content,
        });
    }

    let new_id = conn
        .transaction::<NodeId, diesel::result::Error, _>(|tx| {
            let created_at_ms = now_unix_ms();

            let current_head = head_dsl::head
                .select(head_dsl::node_id)
                .first::<Option<String>>(tx)
                .optional()?
                .flatten();

            let new_id = new_node_id(message_text, current_head.as_deref(), created_at_ms);

            diesel::insert_into(nodes_dsl::nodes)
                .values((
                    nodes_dsl::id.eq(&new_id),
                    nodes_dsl::message.eq(message_text),
                    nodes_dsl::created_at_unix_ms.eq(created_at_ms),
                ))
                .execute(tx)?;

            if let Some(parent_id) = current_head {
                diesel::insert_into(node_parents_dsl::node_parents)
                    .values((
                        node_parents_dsl::node_id.eq(&new_id),
                        node_parents_dsl::parent_id.eq(parent_id),
                        node_parents_dsl::ord.eq(0),
                    ))
                    .execute(tx)?;
            }

            diesel::update(head_dsl::head)
                .set(head_dsl::node_id.eq(Some(new_id.clone())))
                .execute(tx)?;

            use crate::schema::blobs::dsl as blobs_dsl;
            use crate::schema::node_files::dsl as node_files_dsl;

            for file in &snapshot_files {
                diesel::insert_into(blobs_dsl::blobs)
                    .values((
                        blobs_dsl::id.eq(&file.blob_id),
                        blobs_dsl::content.eq(&file.content),
                    ))
                    .on_conflict(blobs_dsl::id)
                    .do_nothing()
                    .execute(tx)?;

                diesel::insert_into(node_files_dsl::node_files)
                    .values((
                        node_files_dsl::node_id.eq(&new_id),
                        node_files_dsl::path.eq(&file.path),
                        node_files_dsl::blob_id.eq(&file.blob_id),
                    ))
                    .execute(tx)?;
            }

            Ok(new_id)
        })
        .map_err(to_io)?;

    Ok(new_id)
}

fn now_unix_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

fn new_node_id(message_text: &str, parent: Option<&str>, created_at_ms: i64) -> NodeId {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update("v1\n");
    hasher.update(message_text.as_bytes());
    hasher.update("\n");
    hasher.update(created_at_ms.to_string().as_bytes());
    hasher.update("\n");

    if let Some(parent_id) = parent {
        hasher.update(parent_id.as_bytes());
    }

    hex::encode(hasher.finalize())
}
