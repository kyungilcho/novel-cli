use std::path::Path;

use crate::{Result, WorkSpaceError};
// SQLite 전용 연결 타입
use diesel::{Connection, sqlite::SqliteConnection};
use diesel_migrations::{EmbeddedMigrations, embed_migrations};
use std::fs;

// 컴파일 시점에 migrations 폴더를 바이너리 안에 포함한다.
// 런타임에 SQL 파일 경로를 따로 들고 다니지 않아도 된다.
pub(crate) const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

// 저장소 메타 디렉토리 이름
const NOVEL_DIR: &str = ".novel";
// 메타 디렉토리 안 SQLite 파일 이름
const VCS_DB_FILE: &str = "vcs.db";

// DB 연결 헬퍼:
// - 루트 경로 canonicalize
// - .novel 디렉토리 생성 보장
// - SQLite 연결 오픈
pub(crate) fn open_connection(root: &Path) -> Result<SqliteConnection> {
    // canonicalize: 상대경로/심볼릭 링크를 실제 절대경로로 정규화
    let canonical_root = root.canonicalize()?;

    // Path::join으로 OS 안전하게 하위 경로 결합
    let meta_dir = canonical_root.join(NOVEL_DIR);
    fs::create_dir_all(&meta_dir)?;

    // 최종 DB 파일 경로(.novel/vcs.db)
    let db_path = meta_dir.join(VCS_DB_FILE);

    // to_string_lossy: 경로를 문자열로 변환(UTF-8이 아니어도 안전 변환)
    let db_url = db_path.to_string_lossy();

    // establish는 &str을 받으므로 as_ref()로 &str로 빌려서 넘긴다
    let conn = SqliteConnection::establish(db_url.as_ref()).map_err(to_connection_error)?;
    Ok(conn)
}

// Diesel 연결 에러 -> 공통 WorkSpaceError 변환
pub(crate) fn to_connection_error(e: diesel::ConnectionError) -> WorkSpaceError {
    WorkSpaceError::Io(std::io::Error::other(e.to_string()))
}

// Display 구현 타입이면 모두 받아서 공통 I/O 에러로 감싼다
// (impl Trait 문법: "이 trait를 만족하는 어떤 타입이든")
pub(crate) fn to_io(e: impl std::fmt::Display) -> WorkSpaceError {
    WorkSpaceError::Io(std::io::Error::other(e.to_string()))
}
