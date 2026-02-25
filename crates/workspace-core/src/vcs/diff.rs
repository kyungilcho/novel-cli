use std::collections::HashMap;

use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection, dsl::select};

use crate::{
    DiffKind, FileDiff, NodeDiff, Result, WorkSpaceError,
    schema::{blobs, node_files},
    vcs::db::{open_connection, to_io},
};

pub fn diff_nodes(root: &std::path::Path, from: &str, to: &str) -> Result<NodeDiff> {
    let mut conn = open_connection(root)?;

    if !node_exists(&mut conn, from)? {
        return Err(WorkSpaceError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("node not found: {}", from),
        )));
    }
    if !node_exists(&mut conn, to)? {
        return Err(WorkSpaceError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("node not found: {}", to),
        )));
    }

    let from_map = load_snapshot_map(&mut conn, from)?;
    let to_map = load_snapshot_map(&mut conn, to)?;

    let mut files = Vec::new();

    let mut paths: Vec<String> = from_map.keys().chain(to_map.keys()).cloned().collect();
    paths.sort();
    paths.dedup();

    for path in paths {
        match (from_map.get(&path), to_map.get(&path)) {
            (None, None) => unreachable!(),
            (None, Some(to_blob)) => {
                files.push(build_file_diff(path, DiffKind::Added, None, Some(to_blob)))
            }
            (Some(from_blob), None) => files.push(build_file_diff(
                path,
                DiffKind::Removed,
                Some(from_blob),
                None,
            )),
            (Some(from_blob), Some(to_blob)) if from_blob != to_blob => {
                files.push(build_file_diff(
                    path,
                    DiffKind::Modified,
                    Some(from_blob),
                    Some(to_blob),
                ));
            }
            _ => {}
        }
    }

    Ok(NodeDiff {
        from: from.to_string(),
        to: to.to_string(),
        files,
    })
}

// return path -> blob_id map for node_id
fn load_snapshot_map(
    conn: &mut SqliteConnection,
    node_id: &str,
) -> Result<HashMap<String, Vec<u8>>> {
    let rows = node_files::dsl::node_files
        .inner_join(blobs::dsl::blobs)
        .filter(node_files::dsl::node_id.eq(node_id))
        .select((node_files::dsl::path, blobs::dsl::content))
        .load::<(String, Vec<u8>)>(conn)
        .map_err(to_io)?;

    Ok(rows.into_iter().collect())
}

fn decode_utf(bytes: &[u8]) -> Option<String> {
    std::str::from_utf8(bytes).ok().map(ToString::to_string)
}

fn normalize_text_for_line_diff(text: &str) -> String {
    let mut normalized = text.replace("\r\n", "\n");
    if !normalized.ends_with('\n') {
        normalized.push('\n');
    }
    normalized
}

fn build_file_diff(
    path: String,
    kind: DiffKind,
    before: Option<&[u8]>,
    after: Option<&[u8]>,
) -> FileDiff {
    let binary = before.is_some_and(is_probably_binary) || after.is_some_and(is_probably_binary);

    if binary {
        return FileDiff {
            path,
            kind,
            before_text: None,
            after_text: None,
            unified: None,
            is_binary: true,
        };
    }

    let before_text = before.and_then(decode_utf);
    let after_text = after.and_then(decode_utf);

    let unified =
        if let (Some(before), Some(after)) = (before_text.as_deref(), after_text.as_deref()) {
            let before_normalized = normalize_text_for_line_diff(before);
            let after_normalized = normalize_text_for_line_diff(after);
            let diff = similar::TextDiff::from_lines(&before_normalized, &after_normalized);
            Some(diff.unified_diff().to_string())
        } else {
            None
        };

    FileDiff {
        path,
        kind,
        before_text,
        after_text,
        unified,
        is_binary: false,
    }
}

fn node_exists(conn: &mut SqliteConnection, id: &str) -> Result<bool> {
    use crate::schema::nodes::dsl as nodes_dsl;
    use diesel::dsl::exists;

    let exists = select(exists(nodes_dsl::nodes.filter(nodes_dsl::id.eq(id))))
        .get_result(conn)
        .map_err(to_io)?;

    Ok(exists)
}

fn is_probably_binary(bytes: &[u8]) -> bool {
    bytes.contains(&0) || std::str::from_utf8(bytes).is_err()
}
