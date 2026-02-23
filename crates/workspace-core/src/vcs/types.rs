use serde::Serialize;

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
