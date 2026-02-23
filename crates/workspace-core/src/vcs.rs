// std::fs: 디렉토리 생성 같은 파일시스템 작업에 사용
// std::path::Path: 문자열이 아닌 "경로 타입"으로 인자를 받기 위해 사용
use std::{fs, path::Path};

use diesel::query_dsl::methods::FilterDsl;
use diesel::select;
// Diesel에서 자주 쓰는 trait/함수(prelude)를 한 번에 가져온다
use diesel::{dsl::exists, prelude::*};
// SQLite 전용 연결 타입
use diesel::sqlite::SqliteConnection;
// 임베디드 마이그레이션 실행을 위한 타입/trait/매크로
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
// Tauri IPC/JSON 직렬화를 위해 struct에 Serialize derive를 붙인다
use serde::Serialize;

// crate 루트(lib.rs)에서 선언한 공통 Result/에러 타입을 가져온다
use crate::{Result, WorkSpaceError};

// 컴파일 시점에 migrations 폴더를 바이너리 안에 포함한다.
// 런타임에 SQL 파일 경로를 따로 들고 다니지 않아도 된다.
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

// 노드 ID 타입 별칭(type alias). 현재는 String이지만 나중에 교체하기 쉽다.
pub type NodeId = String;

// 버전 그래프의 "노드(커밋)"를 표현하는 데이터 구조
#[derive(Debug, Clone, Serialize)]
pub struct VersionNode {
    // 노드 자신 고유 ID
    pub id: NodeId,
    // 부모 노드 ID 목록(merge면 2개 이상 가능)
    pub parents: Vec<NodeId>,
    // 커밋 메시지
    pub message: String,
    // 생성 시각(ms, Unix epoch 기준)
    pub created_at_unix_ms: i64,
}

// 저장소 요약 상태. UI에서 빠르게 상태 표시할 때 사용
#[derive(Debug, Clone, Serialize)]
pub struct RepoState {
    // 커밋이 없을 수 있으므로 Option 사용(None = 아직 HEAD 없음)
    pub head: Option<NodeId>,
    // 전체 노드 개수
    pub node_count: usize,
}

// 저장소 메타 디렉토리 이름
const NOVEL_DIR: &str = ".novel";
// 메타 디렉토리 안 SQLite 파일 이름
const VCS_DB_FILE: &str = "vcs.db";

// 저장소 초기화:
// 1) 연결 열기(필요시 .novel 디렉토리 생성)
// 2) 아직 적용 안 된 migration 실행
pub fn init_repo(root: &Path) -> Result<()> {
    // mut: conn을 migration 실행 시 가변 참조(&mut)로 넘겨야 해서 필요
    let mut conn = open_connection(root)?;

    // map_err(to_io): Diesel 계열 에러를 우리 앱 에러 타입으로 변환
    conn.run_pending_migrations(MIGRATIONS).map_err(to_io)?;

    // 함수 반환 타입이 Result<()> 이므로 성공 시 Ok(())
    Ok(())
}

// 미구현 API 스텁. 인자명 앞에 '_'를 붙여 현재 미사용 경고를 막는다.
pub fn commit(_root: &Path, _message: &str) -> Result<NodeId> {
    Err(not_implemented("commit"))
}

// 로그 조회 API 스텁
pub fn log(_root: &Path) -> Result<Vec<VersionNode>> {
    Err(not_implemented("log"))
}

// 저장소 상태 조회:
// - nodes 개수
// - head의 node_id
pub fn repo_state(root: &Path) -> Result<RepoState> {
    // schema::...::dsl 은 Diesel Query DSL에서 컬럼/테이블 심볼을 쓰기 위한 모듈
    use crate::schema::head::dsl as head_dsl;
    use crate::schema::nodes::dsl as nodes_dsl;

    let mut conn = open_connection(root)?;
    conn.run_pending_migrations(MIGRATIONS).map_err(to_io)?;

    // count() 결과는 SQLite에서 BIGINT(i64)로 받는 게 일반적
    let node_count_i64: i64 = nodes_dsl::nodes
        .count()
        .get_result(&mut conn)
        .map_err(to_io)?;

    // SELECT head.node_id FROM head LIMIT 1
    // .optional(): 결과 row가 없으면 Ok(None)로 처리
    // .flatten(): Option<Option<String>> -> Option<String>로 평탄화
    let head = head_dsl::head
        .select(head_dsl::node_id)
        .first::<Option<String>>(&mut conn)
        .optional()
        .map_err(to_io)?
        .flatten();

    // struct literal 문법으로 필드를 채워 반환
    Ok(RepoState {
        head,
        node_count: node_count_i64 as usize,
    })
}

// 체크아웃 API 스텁
pub fn checkout(root: &Path, node_id: &str) -> Result<()> {
    use crate::schema::head::dsl as head_dsl;
    use crate::schema::nodes::dsl as nodes_dsl;

    let mut conn = open_connection(root)?;
    conn.run_pending_migrations(MIGRATIONS).map_err(to_io)?;

    let node_exists = select(exists(diesel::QueryDsl::filter(
        nodes_dsl::nodes,
        nodes_dsl::id.eq(node_id),
    )))
    .get_result::<bool>(&mut conn)
    .map_err(to_io)?;

    if !node_exists {
        return Err(WorkSpaceError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("node not found: {}", node_id),
        )));
    }

    diesel::update(head_dsl::head)
        .set(head_dsl::node_id.eq(Some(node_id.to_string())))
        .execute(&mut conn)
        .map_err(to_io)?;

    Ok(())
}

// DB 연결 헬퍼:
// - 루트 경로 canonicalize
// - .novel 디렉토리 생성 보장
// - SQLite 연결 오픈
fn open_connection(root: &Path) -> Result<SqliteConnection> {
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
fn to_connection_error(e: diesel::ConnectionError) -> WorkSpaceError {
    WorkSpaceError::Io(std::io::Error::other(e.to_string()))
}

// Display 구현 타입이면 모두 받아서 공통 I/O 에러로 감싼다
// (impl Trait 문법: "이 trait를 만족하는 어떤 타입이든")
fn to_io(e: impl std::fmt::Display) -> WorkSpaceError {
    WorkSpaceError::Io(std::io::Error::other(e.to_string()))
}

// 아직 구현되지 않은 기능에 대해 일관된 Unsupported 에러를 만든다
fn not_implemented(op: &str) -> WorkSpaceError {
    WorkSpaceError::Io(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        format!("{op} is not implemented"),
    ))
}
