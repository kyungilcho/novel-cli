use std::path::Path;

use diesel::{OptionalExtension, QueryDsl, RunQueryDsl};
use diesel_migrations::MigrationHarness;

use crate::{RepoState, Result};

use crate::vcs::db::{MIGRATIONS, open_connection, to_io};

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
