use std::io::ErrorKind;
use std::{fs, path::PathBuf};
use workspace_core::{
    WorkSpaceError, create_file, list_files, read_file, resolve_path, write_file,
};

fn setup() -> (tempfile::TempDir, std::path::PathBuf) {
    let td = tempfile::tempdir().unwrap();
    let root = td.path().to_path_buf();

    (td, root)
}

#[test]
fn resolve_path_blocks_parent_traversal() {
    let (_td, root) = setup();
    let res = resolve_path(&root, "../outside.txt");
    assert!(res.is_err());
}

#[test]
fn list_files_returns_relative_sorted_paths() {
    let (_td, root) = setup();

    fs::write(root.join("b.txt"), "b").unwrap();
    fs::write(root.join("a.txt"), "a").unwrap();
    fs::create_dir(root.join("docs")).unwrap();

    let entries = list_files(&root, ".").unwrap();

    let names: Vec<String> = entries
        .iter()
        .map(|e| e.path.to_string_lossy().into_owned())
        .collect();

    assert_eq!(names, vec!["a.txt", "b.txt", "docs"]);

    assert!(
        entries
            .iter()
            .any(|e| e.path == PathBuf::from("docs") && e.is_dir)
    );
}

#[test]
fn write_then_read_file_roundtrip() {
    let (_td, root) = setup();

    write_file(&root, "memo.txt", "hello").expect("write_failed");
    let content = read_file(&root, "memo.txt").expect("read_failed");
    assert_eq!(content, "hello");
}

#[test]
fn read_files_fails_for_directory() {
    let (_td, root) = setup();
    fs::create_dir(root.join("docs")).unwrap();

    let err = read_file(&root, "docs").unwrap_err();
    assert!(matches!(err, WorkSpaceError::Io(ref e) if e.kind() == ErrorKind::InvalidInput));
}

#[test]
fn create_file_works() {
    let (_td, root) = setup();

    let rel_path = create_file(&root, ".", "memo.txt").unwrap();
    assert_eq!(rel_path, PathBuf::from("memo.txt"));
}

#[test]
fn create_file_fails_for_invalid_name() {
    let (_td, root) = setup();

    let err = create_file(&root, ".", "../memo.txt").unwrap_err();
    assert!(matches!(err, WorkSpaceError::InvalidFileName(ref name) if name == "../memo.txt"));
}

#[test]
fn create_file_fails_for_duplicated_name() {
    let (_td, root) = setup();

    create_file(&root, ".", "memo.txt").unwrap();
    let err = create_file(&root, ".", "memo.txt").unwrap_err();
    assert!(matches!(err, WorkSpaceError::Io(ref e) if e.kind() == ErrorKind::AlreadyExists));
}
