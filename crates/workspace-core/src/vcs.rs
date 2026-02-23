use std::path::PathBuf;
// std::fs: 디렉토리 생성 같은 파일시스템 작업에 사용
// std::path::Path: 문자열이 아닌 "경로 타입"으로 인자를 받기 위해 사용
use std::{fs, path::Path};

use diesel::select;
// Diesel에서 자주 쓰는 trait/함수(prelude)를 한 번에 가져온다
use diesel::{dsl::exists, prelude::*};
// SQLite 전용 연결 타입
use diesel::sqlite::SqliteConnection;
// 임베디드 마이그레이션 실행을 위한 타입/trait/매크로
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
// Tauri IPC/JSON 직렬화를 위해 struct에 Serialize derive를 붙인다
use serde::Serialize;

use crate::schema::node_parents;
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

pub fn commit(root: &Path, message: &str) -> Result<NodeId> {
    use crate::schema::head::dsl as head_dsl;
    use crate::schema::node_parents::dsl as node_parents_dsl;
    use crate::schema::nodes::dsl as nodes_dsl;

    let message_text = message.trim();

    if message_text.is_empty() {
        return Err(WorkSpaceError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "empty commit message",
        )));
    }

    let mut conn = open_connection(root)?;
    conn.run_pending_migrations(MIGRATIONS).map_err(to_io)?;

    let files = collect_files_in_workspace(root)?;
    let mut snapshot_files = Vec::with_capacity(files.len());

    for rel in files {
        let abs = root.join(&rel);

        let content = fs::read(&abs).map_err(to_io)?;

        let blob_id = blob_id_for_content(&content);

        snapshot_files.push(SnapshotFile {
            path: normalize_rel_path(&rel),
            blob_id,
            content,
        });
    }

    let new_id = conn
        .transaction::<NodeId, diesel::result::Error, _>(|tx| {
            let created_at_ms = now_unix_ms();

            let current_head = head_dsl::head
                .select(head_dsl::node_id)
                .first::<Option<String>>(tx)
                .optional()?
                .flatten();

            let new_id = new_node_id(message_text, current_head.as_deref(), created_at_ms);

            diesel::insert_into(nodes_dsl::nodes)
                .values((
                    nodes_dsl::id.eq(&new_id),
                    nodes_dsl::message.eq(message_text),
                    nodes_dsl::created_at_unix_ms.eq(created_at_ms),
                ))
                .execute(tx)?;

            if let Some(parent_id) = current_head {
                diesel::insert_into(node_parents_dsl::node_parents)
                    .values((
                        node_parents_dsl::node_id.eq(&new_id),
                        node_parents_dsl::parent_id.eq(parent_id),
                        node_parents_dsl::ord.eq(0),
                    ))
                    .execute(tx)?;
            }

            diesel::update(head_dsl::head)
                .set(head_dsl::node_id.eq(Some(new_id.clone())))
                .execute(tx)?;

            use crate::schema::blobs::dsl as blobs_dsl;
            use crate::schema::node_files::dsl as node_files_dsl;

            for file in &snapshot_files {
                diesel::insert_into(blobs_dsl::blobs)
                    .values((
                        blobs_dsl::id.eq(&file.blob_id),
                        blobs_dsl::content.eq(&file.content),
                    ))
                    .on_conflict(blobs_dsl::id)
                    .do_nothing()
                    .execute(tx)?;

                diesel::insert_into(node_files_dsl::node_files)
                    .values((
                        node_files_dsl::node_id.eq(&new_id),
                        node_files_dsl::path.eq(&file.path),
                        node_files_dsl::blob_id.eq(&file.blob_id),
                    ))
                    .execute(tx)?;
            }

            Ok(new_id)
        })
        .map_err(to_io)?;

    Ok(new_id)
}

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
pub fn checkout(root: &Path, target_node_id: &str) -> Result<()> {
    use crate::schema::blobs::dsl as blobs_dsl;
    use crate::schema::head::dsl as head_dsl;
    use crate::schema::node_files::dsl as node_files_dsl;
    use crate::schema::nodes::dsl as nodes_dsl;

    let mut conn = open_connection(root)?;
    conn.run_pending_migrations(MIGRATIONS).map_err(to_io)?;

    let node_exists = select(exists(diesel::QueryDsl::filter(
        nodes_dsl::nodes,
        nodes_dsl::id.eq(target_node_id),
    )))
    .get_result::<bool>(&mut conn)
    .map_err(to_io)?;

    if !node_exists {
        return Err(WorkSpaceError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("node not found: {}", target_node_id),
        )));
    }

    let rows = node_files_dsl::node_files
        .inner_join(blobs_dsl::blobs.on(node_files_dsl::blob_id.eq(blobs_dsl::id)))
        .filter(node_files_dsl::node_id.eq(target_node_id))
        .select((node_files_dsl::path, blobs_dsl::content))
        .load::<(String, Vec<u8>)>(&mut conn)
        .map_err(to_io)?;

    let canonical_root = root.canonicalize()?;
    let current = collect_files_in_workspace(&canonical_root)?
        .into_iter()
        .map(|p| normalize_rel_path(&p))
        .collect::<std::collections::HashSet<_>>();

    let target = rows
        .iter()
        .map(|(p, _)| p.clone())
        .collect::<std::collections::HashSet<_>>();

    for rel in current.difference(&target) {
        let abs = canonical_root.join(rel);
        fs::remove_file(abs)?;
    }

    for (rel, content) in rows {
        let abs = canonical_root.join(rel);

        if let Some(parent) = abs.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(abs, content)?;
    }

    diesel::update(head_dsl::head)
        .set(head_dsl::node_id.eq(Some(target_node_id.to_string())))
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
fn new_node_id(msg_text: &str, parent: Option<&str>, created_at_ms: i64) -> NodeId {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();

    hasher.update("v1\n");
    hasher.update(msg_text.as_bytes());
    hasher.update("\n");
    hasher.update(created_at_ms.to_string().as_bytes());
    hasher.update("\n");
    if let Some(p) = parent {
        hasher.update(p.as_bytes());
    }
    hex::encode(hasher.finalize())
}

fn now_unix_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
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

fn collect_files_in_workspace(root: &Path) -> Result<Vec<PathBuf>> {
    let canonical_root = root.canonicalize()?;
    let mut out = Vec::new();
    collect_files_recursive(&canonical_root, &canonical_root, &mut out)?;
    out.sort();
    Ok(out)
}

#[derive(Debug, Clone)]
struct SnapshotFile {
    path: String,
    blob_id: String,
    content: Vec<u8>,
}

fn normalize_rel_path(p: &Path) -> String {
    p.to_string_lossy().replace("\\", "/")
}

fn blob_id_for_content(content: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(b"blob ");
    hasher.update(content);
    hex::encode(hasher.finalize())
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffKind {
    Added,
    Removed,
    Modified,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FileDiff {
    pub path: String,
    pub kind: DiffKind,
    pub before_text: Option<String>,
    pub after_text: Option<String>,
    pub unified: Option<String>,
    pub is_binary: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct NodeDiff {
    pub from: String,
    pub to: String,
    pub files: Vec<FileDiff>,
}

pub fn diff_nodes(_root: &std::path::Path, _from: &str, _to: &str) -> Result<NodeDiff> {
    todo!("implement diff_nodes");
}
