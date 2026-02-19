use assert_cmd::cargo;
use predicates::prelude::*;
use std::path::Path;
use tempfile::tempdir;

fn run_in(dir: &Path, args: &[&str]) -> assert_cmd::assert::Assert {
    let mut cmd = cargo::cargo_bin_cmd!("novel-cli");

    cmd.current_dir(dir).args(args).assert()
}

#[test]
fn add_and_list_persists_notes() {
    let dir = tempdir().unwrap();
    let run = |args: &[&str]| run_in(dir.path(), args);

    run(&["add", "first"])
        .success()
        .stdout(predicate::str::contains("adding Note #1"));

    run(&["add", "second"])
        .success()
        .stdout(predicate::str::contains("adding Note #2"));

    run(&["list"]).success().stdout(
        predicate::str::contains("[ ] 1: first").and(predicate::str::contains("[ ] 2: second")),
    );
}

#[test]
fn done_edit_remove_flow() {
    let dir = tempdir().unwrap();
    let run = |args: &[&str]| run_in(dir.path(), args);

    run(&["add", "task"]).success();
    run(&["done", "1"])
        .success()
        .stdout(predicate::str::contains("marked Note #1 as done"));

    run(&["edit", "1", "updated"]).success();
    run(&["list"])
        .success()
        .stdout(predicate::str::contains("[x] 1: updated"));

    // remove
    // check whether the note is removed(empty list)
    run(&["remove", "1"]).success();
    run(&["list"])
        .success()
        .stdout(predicate::str::contains("No notes found."));
}

#[test]
fn invalid_id() {
    let dir = tempdir().unwrap();

    run_in(dir.path(), &["done", "999"])
        .failure()
        .stderr(predicate::str::contains("error: Invalid ID: 999"));
}

#[test]
fn filter_notes() {
    let dir = tempdir().unwrap();
    let run = |args: &[&str]| run_in(dir.path(), args);

    run(&["add", "first"]).success();
    run(&["add", "second"]).success();
    run(&["add", "third"]).success();

    run(&["done", "2"]).success();

    run(&["list"]).success().stdout(
        predicate::str::contains("[ ] 1: first")
            .and(predicate::str::contains("[x] 2: second"))
            .and(predicate::str::contains("[ ] 3: third")),
    );

    run(&["list", "--done"]).success().stdout(
        predicate::str::contains("[x] 2: second")
            .and(predicate::str::contains("[ ] 1: first").not())
            .and(predicate::str::contains("[ ] 3: third").not()),
    );

    run(&["list", "--todo"]).success().stdout(
        predicate::str::contains("[ ] 1: first")
            .and(predicate::str::contains("[x] 2: second").not())
            .and(predicate::str::contains("[ ] 3: third")),
    );

    run(&["list", "--contains", "second"]).success().stdout(
        predicate::str::contains("[x] 2: second")
            .and(predicate::str::contains("[ ] 1: first").not())
            .and(predicate::str::contains("[ ] 3: third").not()),
    );

    run(&["list", "--done", "--contains", "second"])
        .success()
        .stdout(
            predicate::str::contains("[x] 2: second")
                .and(predicate::str::contains("[ ] 1: first").not())
                .and(predicate::str::contains("[ ] 3: third").not()),
        );

    run(&["list", "--todo", "--contains", "second"])
        .success()
        .stdout(predicate::str::contains("No notes found."));
}
