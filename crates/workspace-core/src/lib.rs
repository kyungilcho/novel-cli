use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use serde::Serialize;

pub mod vcs;
pub use vcs::*;
pub mod schema;

#[derive(Debug, Clone, Serialize)]
pub struct ProjectInfo {
    pub root: PathBuf,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileEntry {
    pub path: PathBuf,
    pub is_dir: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum WorkSpaceError {
    #[error("invalid root: {0}")]
    InvalidRoot(PathBuf),

    #[error("path escape: {0}")]
    PathEscape(PathBuf),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("path is outside root: {0}")]
    PathOutsideRoot(std::path::PathBuf),

    #[error("invalid file name: {0}")]
    InvalidFileName(String),
}

pub type Result<T> = std::result::Result<T, WorkSpaceError>;

pub fn open_project(root: &Path) -> Result<ProjectInfo> {
    if !root.exists() || !root.is_dir() {
        return Err(WorkSpaceError::InvalidRoot(root.to_path_buf()));
    }

    let canonical = root.canonicalize()?;
    let name = canonical
        .file_name()
        .unwrap_or(OsStr::new("project"))
        .to_string_lossy()
        .to_string();

    Ok(ProjectInfo {
        root: canonical,
        name,
    })
}

pub fn resolve_path(root: &Path, rel: &str) -> Result<PathBuf> {
    let canonicalized_root = root.canonicalize()?;
    let joined = canonicalized_root.join(rel);

    let candidate = joined.canonicalize()?;

    if !candidate.starts_with(&canonicalized_root) {
        return Err(WorkSpaceError::PathEscape(joined));
    }

    Ok(candidate)
}

pub fn list_files(root: &Path, rel: &str) -> Result<Vec<FileEntry>> {
    let dir = resolve_path(root, rel)?;

    let mut entries = Vec::new();

    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        let is_dir = path.is_dir();
        entries.push(FileEntry {
            path: path
                .strip_prefix(root.canonicalize()?)
                .map_err(|_| WorkSpaceError::PathOutsideRoot(path.clone()))?
                .to_path_buf(),
            is_dir,
        });
    }

    entries.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(entries)
}

pub fn read_file(root: &Path, rel: &str) -> Result<String> {
    let path = resolve_path(root, rel)?;

    if !path.is_file() {
        return Err(WorkSpaceError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "path is not a file",
        )));
    }

    Ok(std::fs::read_to_string(&path)?)
}

pub fn write_file(root: &Path, rel: &str, content: &str) -> Result<()> {
    let canonicalized_root = root.canonicalize()?;
    let path = canonicalized_root.join(rel);

    let parent = path.parent().ok_or(WorkSpaceError::Io(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        "path has no parent",
    )))?;

    let canonicalized_parent = parent.canonicalize()?;
    if !canonicalized_parent.starts_with(&canonicalized_root) {
        return Err(WorkSpaceError::PathEscape(path));
    }

    if path.exists() && !path.is_file() {
        return Err(WorkSpaceError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "path is not a file",
        )));
    }

    std::fs::write(&path, content)?;
    Ok(())
}

pub fn create_file(root: &Path, rel: &str, name: &str) -> Result<PathBuf> {
    if name.is_empty() || name.contains('/') || name.contains('\\') || name == "." || name == ".." {
        return Err(WorkSpaceError::InvalidFileName(name.to_string()));
    }

    let canonicalized_root = root.canonicalize()?;

    let parent_path = resolve_path(&canonicalized_root, rel)?;

    if !parent_path.is_dir() {
        return Err(WorkSpaceError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "path is not a directory",
        )));
    }

    let path = parent_path.join(name);

    let rel = path
        .strip_prefix(&canonicalized_root)
        .map_err(|_| WorkSpaceError::PathOutsideRoot(path.clone()))?
        .to_path_buf();

    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;

    Ok(rel)
}
