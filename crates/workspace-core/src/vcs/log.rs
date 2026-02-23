use std::path::Path;

use diesel::prelude::*;
use diesel_migrations::MigrationHarness;

use crate::schema::node_parents;
use crate::vcs::db::{MIGRATIONS, open_connection, to_io};
use crate::{Result, VersionNode};

#[derive(Debug, Queryable)]
pub struct NodeRow {
    pub id: String,
    pub message: String,
    pub created_at_unix_ms: i64,
}

// 로그 조회 API 스텁
pub fn log(root: &Path) -> Result<Vec<VersionNode>> {
    use crate::schema::nodes::dsl as nodes_dsl;

    let mut conn = open_connection(root)?;
    conn.run_pending_migrations(MIGRATIONS).map_err(to_io)?;

    let node_rows = nodes_dsl::nodes
        .select((
            nodes_dsl::id,
            nodes_dsl::message,
            nodes_dsl::created_at_unix_ms,
        ))
        .order(nodes_dsl::created_at_unix_ms.desc())
        .load::<NodeRow>(&mut conn)
        .map_err(to_io)?;

    let mut out: Vec<VersionNode> = Vec::with_capacity(node_rows.len());

    for row in node_rows {
        let parents = node_parents::dsl::node_parents
            .filter(node_parents::dsl::node_id.eq(&row.id))
            .select(node_parents::dsl::parent_id)
            .order(node_parents::dsl::ord.asc())
            .load::<String>(&mut conn)
            .map_err(to_io)?;

        out.push(VersionNode {
            id: row.id,
            message: row.message,
            created_at_unix_ms: row.created_at_unix_ms,
            parents,
        });
    }

    Ok(out)
}
