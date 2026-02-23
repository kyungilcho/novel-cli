use std::path::{Path, PathBuf};

use crate::{Result, WorkSpaceError};

#[derive(Debug, Clone)]
pub(crate) struct SnapshotFile {
    pub path: String,
    pub blob_id: String,
    pub content: Vec<u8>,
}

pub(crate) fn blob_id_for_content(content: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(b"blob ");
    hasher.update(content);
    hex::encode(hasher.finalize())
}

fn collect_files_recursive(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;

        let path = entry.path();

        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            if path.file_name().and_then(|n| n.to_str()) == Some(".novel") {
                continue;
            }
            collect_files_recursive(root, &path, out)?;
        }

        if file_type.is_file() {
            let rel = path
                .strip_prefix(root)
                .map_err(|_| WorkSpaceError::PathOutsideRoot(path.clone()))?;
            out.push(rel.to_path_buf());
        }
    }

    Ok(())
}

pub(crate) fn collect_files_in_workspace(root: &Path) -> Result<Vec<PathBuf>> {
    let canonical_root = root.canonicalize()?;
    let mut out = Vec::new();
    collect_files_recursive(&canonical_root, &canonical_root, &mut out)?;
    out.sort();
    Ok(out)
}

pub(crate) fn normalize_rel_path(p: &Path) -> String {
    p.to_string_lossy().replace("\\", "/")
}
