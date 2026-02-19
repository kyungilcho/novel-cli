use assert_cmd::cargo;
use predicates::prelude::*;
use std::path::Path;
use tempfile::tempdir;

use diesel::prelude::*;
use diesel::sql_query;
use diesel::sqlite::SqliteConnection;

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
        predicate::str::contains("[ ] P0: 1 first")
            .and(predicate::str::contains("[ ] P0: 2 second")),
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
        .stdout(predicate::str::contains("[x] P0: 1 updated"));

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
        predicate::str::contains("[ ] P0: 1 first")
            .and(predicate::str::contains("[x] P0: 2 second"))
            .and(predicate::str::contains("[ ] P0: 3 third")),
    );

    run(&["list", "--done"]).success().stdout(
        predicate::str::contains("[x] P0: 2 second")
            .and(predicate::str::contains("[ ] P0: 1 first").not())
            .and(predicate::str::contains("[ ] P0: 3 third").not()),
    );

    run(&["list", "--todo"]).success().stdout(
        predicate::str::contains("[ ] P0: 1 first")
            .and(predicate::str::contains("[x] P0: 2 second").not())
            .and(predicate::str::contains("[ ] P0: 3 third")),
    );

    run(&["list", "--contains", "second"]).success().stdout(
        predicate::str::contains("[x] P0: 2 second")
            .and(predicate::str::contains("[ ] P0: 1 first").not())
            .and(predicate::str::contains("[ ] P0: 3 third").not()),
    );

    run(&["list", "--done", "--contains", "second"])
        .success()
        .stdout(
            predicate::str::contains("[x] P0: 2 second")
                .and(predicate::str::contains("[ ] P0: 1 first").not())
                .and(predicate::str::contains("[ ] P0: 3 third").not()),
        );

    run(&["list", "--todo", "--contains", "second"])
        .success()
        .stdout(predicate::str::contains("No notes found."));
}

#[test]
fn priority_works() {
    let dir = tempdir().unwrap();
    let run = |args: &[&str]| run_in(dir.path(), args);

    run(&["add", "first", "--priority", "1"]).success();

    run(&["list"])
        .success()
        .stdout(predicate::str::contains("[ ] P1: 1 first"));

    run(&["list", "--priority", "1"])
        .success()
        .stdout(predicate::str::contains("[ ] P1: 1 first"));
}

#[test]
fn priority_command_updates_and_rejects_out_of_range() {
    let dir = tempdir().unwrap();
    let run = |args: &[&str]| run_in(dir.path(), args);

    run(&["add", "first"]).success();
    run(&["priority", "1", "4"])
        .success()
        .stdout(predicate::str::contains("set priority of Note #1 to 4"));
    run(&["list"])
        .success()
        .stdout(predicate::str::contains("[ ] P4: 1 first"));

    run_in(dir.path(), &["add", "bad", "--priority", "9"])
        .failure()
        .stderr(predicate::str::contains("error: Invalid priority: 9"));

    run_in(dir.path(), &["priority", "1", "9"])
        .failure()
        .stderr(predicate::str::contains("error: Invalid priority: 9"));
}

fn seed_v1_db(dir: &Path) {
    let db_path = dir.join("notes.db");
    let mut conn = SqliteConnection::establish(db_path.to_str().expect("valid db path")).unwrap();

    sql_query(
        "CREATE TABLE notes (
            id INTEGER PRIMARY KEY,
            text TEXT NOT NULL,
            done INTEGER NOT NULL CHECK (done IN (0, 1))
        )",
    )
    .execute(&mut conn)
    .unwrap();

    sql_query("INSERT INTO notes (id, text, done) VALUES (1, 'legacy task', 0)")
        .execute(&mut conn)
        .unwrap();
}

#[test]
fn migrates_v1_db_on_first_run() {
    let dir = tempdir().unwrap();
    let run = |args: &[&str]| run_in(dir.path(), args);

    // old schema DB 생성
    seed_v1_db(dir.path());

    // 첫 실행에서 마이그레이션 + 기존 데이터 유지 확인
    run(&["list"])
        .success()
        .stdout(predicate::str::contains("[ ] P0: 1 legacy task"));

    // 마이그레이션 이후 쓰기 동작도 정상인지 확인
    run(&["add", "fresh task"]).success();

    run(&["list"]).success().stdout(
        predicate::str::contains("[ ] P0: 1 legacy task")
            .and(predicate::str::contains("[ ] P0: 2 fresh task")),
    );
}
