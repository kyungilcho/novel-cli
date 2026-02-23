use std::fs;
use std::path::Path;

use diesel::{dsl::exists, prelude::*, select};
use diesel_migrations::MigrationHarness;

use crate::vcs::db::{MIGRATIONS, open_connection, to_io};
use crate::vcs::snapshot::{collect_files_in_workspace, normalize_rel_path};
use crate::{Result, WorkSpaceError};

// 체크아웃 API 스텁
pub fn checkout(root: &Path, target_node_id: &str) -> Result<()> {
    use crate::schema::blobs::dsl as blobs_dsl;
    use crate::schema::head::dsl as head_dsl;
    use crate::schema::node_files::dsl as node_files_dsl;
    use crate::schema::nodes::dsl as nodes_dsl;

    let mut conn = open_connection(root)?;
    conn.run_pending_migrations(MIGRATIONS).map_err(to_io)?;

    let node_exists = select(exists(diesel::QueryDsl::filter(
        nodes_dsl::nodes,
        nodes_dsl::id.eq(target_node_id),
    )))
    .get_result::<bool>(&mut conn)
    .map_err(to_io)?;

    if !node_exists {
        return Err(WorkSpaceError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("node not found: {}", target_node_id),
        )));
    }

    let rows = node_files_dsl::node_files
        .inner_join(blobs_dsl::blobs.on(node_files_dsl::blob_id.eq(blobs_dsl::id)))
        .filter(node_files_dsl::node_id.eq(target_node_id))
        .select((node_files_dsl::path, blobs_dsl::content))
        .load::<(String, Vec<u8>)>(&mut conn)
        .map_err(to_io)?;

    let canonical_root = root.canonicalize()?;
    let current = collect_files_in_workspace(&canonical_root)?
        .into_iter()
        .map(|p| normalize_rel_path(&p))
        .collect::<std::collections::HashSet<_>>();

    let target = rows
        .iter()
        .map(|(p, _)| p.clone())
        .collect::<std::collections::HashSet<_>>();

    for rel in current.difference(&target) {
        let abs = canonical_root.join(rel);
        fs::remove_file(abs)?;
    }

    for (rel, content) in rows {
        let abs = canonical_root.join(rel);

        if let Some(parent) = abs.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(abs, content)?;
    }

    diesel::update(head_dsl::head)
        .set(head_dsl::node_id.eq(Some(target_node_id.to_string())))
        .execute(&mut conn)
        .map_err(to_io)?;

    Ok(())
}
