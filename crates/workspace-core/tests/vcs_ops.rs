use workspace_core::{DiffKind, Result, checkout, commit, diff_nodes, init_repo, log, repo_state};

fn setup() -> (tempfile::TempDir, std::path::PathBuf) {
    let td = tempfile::tempdir().unwrap();
    let root = td.path().to_path_buf();

    (td, root)
}

#[test]
fn init_repo_creates_state() -> Result<()> {
    let (_td, root) = setup();

    init_repo(&root)?;

    let state = repo_state(&root)?;

    assert_eq!(state.node_count, 0);
    assert!(state.head.is_none());

    Ok(())
}

#[test]
fn commit_sets_head_and_increments_count() -> Result<()> {
    let (_td, root) = setup();

    init_repo(&root)?;

    let first_commit_id = commit(&root, "initial commit")?;

    let state = repo_state(&root)?;

    assert_eq!(state.node_count, 1);
    assert_eq!(state.head, Some(first_commit_id));

    Ok(())
}

#[test]
fn second_commit_links_first_as_parent() -> Result<()> {
    let (_td, root) = setup();

    init_repo(&root)?;

    let first_commit_id = commit(&root, "initial commit")?;
    let second_commit_id = commit(&root, "second commit")?;

    let nodes = log(&root)?;

    let second_node = nodes.iter().find(|n| n.id == second_commit_id).unwrap();

    assert_eq!(second_node.parents, vec![first_commit_id]);

    Ok(())
}

#[test]
fn commit_rejects_empty_message() -> Result<()> {
    let (_td, root) = setup();

    init_repo(&root)?;

    let first_commit_err = commit(&root, "").unwrap_err();

    assert!(
        matches!(first_commit_err, workspace_core::WorkSpaceError::Io(ref e) if e.kind() == std::io::ErrorKind::InvalidInput)
    );

    Ok(())
}

#[test]
fn checkout_restores_file_contents() -> Result<()> {
    let (_td, root) = setup();

    init_repo(&root)?;

    let file_path = root.join("hello.txt");
    std::fs::write(&file_path, "hello world")?;

    let first_commit_id = commit(&root, "initial commit")?;

    std::fs::remove_file(&file_path)?;

    assert!(!file_path.exists());

    commit(&root, "second commit")?;

    checkout(&root, &first_commit_id)?;

    let content = std::fs::read_to_string(&file_path)?;
    assert_eq!(content, "hello world");

    Ok(())
}

#[test]
fn checkout_removes_files_not_in_target_snapshot() -> Result<()> {
    let (_td, root) = setup();

    init_repo(&root)?;

    let first_commit_id = commit(&root, "initial commit")?;

    let file_a = root.join("a.txt");

    std::fs::write(&file_a, "hello")?;

    commit(&root, "second commit")?;

    assert!(file_a.exists());

    checkout(&root, &first_commit_id)?;

    assert!(!file_a.exists());

    Ok(())
}

#[test]
fn diff_nodes_return_added() -> Result<()> {
    let (_td, root) = setup();

    init_repo(&root)?;

    let first_commit_id = commit(&root, "initial commit")?;

    let file_path = root.join("hello.txt");
    std::fs::write(&file_path, "hello world")?;

    let second_commit_id = commit(&root, "second commit")?;

    let diff = diff_nodes(&root, &first_commit_id, &second_commit_id)?;

    assert_eq!(diff.files.len(), 1);
    assert_eq!(diff.files[0].kind, DiffKind::Added);
    assert_eq!(diff.files[0].path, "hello.txt");
    assert_eq!(diff.files[0].before_text, None);
    assert_eq!(diff.files[0].after_text, Some("hello world".to_string()));

    Ok(())
}

#[test]
fn diff_nodes_return_removed() -> Result<()> {
    let (_td, root) = setup();

    init_repo(&root)?;

    let file_path = root.join("hello.txt");
    std::fs::write(&file_path, "hello world")?;

    let first_commit_id = commit(&root, "initial commit")?;

    std::fs::remove_file(&file_path)?;

    let second_commit_id = commit(&root, "second commit")?;

    let diff = diff_nodes(&root, &first_commit_id, &second_commit_id)?;

    assert_eq!(diff.files.len(), 1);
    assert_eq!(diff.files[0].kind, DiffKind::Removed);
    assert_eq!(diff.files[0].path, "hello.txt");
    assert_eq!(diff.files[0].before_text, Some("hello world".to_string()));
    assert_eq!(diff.files[0].after_text, None);

    Ok(())
}

#[test]
fn diff_nodes_return_modified() -> Result<()> {
    let (_td, root) = setup();

    init_repo(&root)?;

    let file_path = root.join("hello.txt");
    std::fs::write(&file_path, "hello world")?;

    let first_commit_id = commit(&root, "initial commit")?;

    std::fs::write(&file_path, "hello world modified")?;

    let second_commit_id = commit(&root, "second commit")?;

    let diff = diff_nodes(&root, &first_commit_id, &second_commit_id)?;

    assert_eq!(diff.files.len(), 1);
    assert_eq!(diff.files[0].kind, DiffKind::Modified);
    assert_eq!(diff.files[0].path, "hello.txt");
    assert_eq!(diff.files[0].before_text, Some("hello world".to_string()));
    assert_eq!(
        diff.files[0].after_text,
        Some("hello world modified".to_string())
    );
    assert_eq!(diff.files[0].unified, Some("@@ -1 +1 @@\n-hello world\n\\ No newline at end of file\n+hello world modified\n\\ No newline at end of file\n".to_string()));

    Ok(())
}

#[test]
fn diff_nodes_return_no_diff() -> Result<()> {
    let (_td, root) = setup();

    init_repo(&root)?;

    let file_path = root.join("hello.txt");
    std::fs::write(&file_path, "hello world")?;

    let first_commit_id = commit(&root, "initial commit")?;

    let second_commit_id = commit(&root, "second commit")?;

    let diff = diff_nodes(&root, &first_commit_id, &second_commit_id)?;

    assert_eq!(diff.files.len(), 0);

    Ok(())
}

#[test]
fn diff_nodes_rejects_nonexistent_to_node() -> Result<()> {
    let (_td, root) = setup();

    init_repo(&root)?;

    let first_commit_id = commit(&root, "initial commit")?;

    let diff_error = diff_nodes(&root, &first_commit_id, "nonexistent").unwrap_err();

    assert!(
        matches!(diff_error, workspace_core::WorkSpaceError::Io(ref e) if e.kind() == std::io::ErrorKind::NotFound)
    );

    Ok(())
}

#[test]
fn diff_nodes_rejects_nonexistent_from_node() -> Result<()> {
    let (_td, root) = setup();

    init_repo(&root)?;

    let first_commit_id = commit(&root, "initial commit")?;

    let diff_error = diff_nodes(&root, "nonexistent", &first_commit_id).unwrap_err();

    assert!(
        matches!(diff_error, workspace_core::WorkSpaceError::Io(ref e) if e.kind() == std::io::ErrorKind::NotFound)
    );

    Ok(())
}

#[test]
fn binary_file_diff_is_detected() -> Result<()> {
    let (_td, root) = setup();

    init_repo(&root)?;

    let file_path = root.join("binary.bin");
    std::fs::write(&file_path, &[0u8, 1, 2, 3])?;

    let first_commit_id = commit(&root, "initial commit")?;

    std::fs::write(&file_path, &[0u8, 1, 2, 3, 4])?;

    let second_commit_id = commit(&root, "second commit")?;

    let diff = diff_nodes(&root, &first_commit_id, &second_commit_id)?;

    assert_eq!(diff.files.len(), 1);
    assert_eq!(diff.files[0].kind, DiffKind::Modified);
    assert_eq!(diff.files[0].path, "binary.bin");
    assert_eq!(diff.files[0].before_text, None);
    assert_eq!(diff.files[0].after_text, None);
    assert_eq!(diff.files[0].unified, None);

    Ok(())
}
