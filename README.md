# Novel IDE (Working Title)

소설 집필을 위한 데스크톱 IDE를 목표로 하는 프로젝트다.  
핵심 방향은 다음 3가지다.

- 소설 집필에 최적화된 툴
- Git 기반 그래프형 버전 관리
- VS Code 같은 작업 경험(파일 트리, 탭, 편집기, 소스 컨트롤)

## Vision

일반 메모 앱이 아니라, 집필 흐름(초안 작성, 분기, 비교, 되돌리기)에 맞춘 IDE를 만든다.  
문서 단위 저장을 넘어, 이야기의 분기와 합류를 명시적으로 다룰 수 있어야 한다.

## Core Concepts

- Project: 하나의 소설 작업 단위
- Chapter: 장(큰 구조)
- Scene: 장 안의 세부 단위
- Snapshot: 특정 시점의 저장본(= Commit)
- Branch: 서사 분기(대체 전개, 실험 버전)

## Current Repository Structure

```text
.
├── apps/
│   ├── cli/             # CLI app
│   └── desktop/         # Tauri + React app
├── crates/
│   └── core/            # domain/storage logic
├── Cargo.toml           # Rust workspace
├── package.json         # pnpm workspace scripts
└── pnpm-workspace.yaml
```

## MVP Scope

포함:

- 파일 트리 + 탭 기반 편집
- 자동 저장/수동 저장
- Git commit/branch/checkout
- Commit 그래프 시각화
- Commit 간 diff 확인

제외(초기):

- 실시간 협업
- 클라우드 동기화
- AI 자동 집필/자동 수정

## Architecture Direction

- Frontend: React + Monaco Editor
- Desktop Runtime: Tauri
- Backend(Core): Rust
- Versioning: Git (Rust에서 명령/라이브러리 래핑)
- Metadata: SQLite (프로젝트 메타데이터, UI 상태 등)

## Development

Prerequisites:

- Rust toolchain
- Node.js + pnpm
- Tauri prerequisites (OS별)

Install:

```bash
pnpm install
```

Rust workspace check:

```bash
cargo check --workspace
```

Desktop dev:

```bash
pnpm tauri:dev
```

Type check:

```bash
pnpm typecheck
```

CLI test:

```bash
cargo test -p novel-cli
```

## Roadmap

Phase 1:

- 안정적인 파일 편집/저장 루프 구축
- 프로젝트 열기/파일 탐색 UX 정리

Phase 2:

- Git commit/branch/checkout를 UI에서 실행
- 변경 상태(Modified, Staged 등) 표시

Phase 3:

- Commit DAG 그래프 뷰
- 선택 커밋 기준 diff/restore

Phase 4:

- 소설 전용 기능(장면 태그, 등장인물 링크, 플롯 체크)
- 집필 생산성 기능(단축키/명령 팔레트/검색 강화)

## License

MIT (see `LICENSE`)
