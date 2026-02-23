# Desktop Design Reference (Stitch Prototype)

이 문서는 사용자가 제공한 Stitch HTML 시안을 기준으로, 현재 `apps/desktop` 구현이 따라야 할 UI 규칙을 정리한다.

## Goal

- VS Code 스타일의 dense editor UX
- 소설 집필 도메인에 맞춘 Source Control + 파일 탐색 동선
- 다크 테마 기반의 안정적 대비

## Shell Layout

기본 레이아웃은 4개 영역으로 고정한다.

1. Activity Bar (`50px`)
2. Sidebar (`250px`, Explorer/History 전환)
3. Workspace (`tabs + breadcrumb + editor + minimap`)
4. Status Bar (`22px`)

`apps/desktop/src/styles/workspace.css`의 `.app-shell` grid가 이 구조를 담당한다.

## Color & Typography Tokens

시안 기준 핵심 토큰:

- `--color-primary: #0a78c2`
- `--color-bg-app: #101b22`
- `--color-bg-editor: #1e1e1e`
- `--color-bg-sidebar: #252526`
- `--color-bg-activity: #333333`
- `--color-text: #d4d4d4`
- `--font-ui: Inter`
- `--font-mono: JetBrains Mono`

토큰 정의 파일:

- `apps/desktop/src/styles/tokens.css`

## Component Intent

### Activity Bar

- 아이콘 중심 네비게이션
- 선택 상태는 좌측 `2px` primary border
- `History` 아이템은 dot indicator로 변경 가능 상태 표시

### Sidebar

- `Explorer`: 파일 열기/생성/루트 변경
- `History`: commit message 입력, commit 실행, snapshot log + checkout

### Editor Workspace

- 탭: active/inactive 대비 명확화
- breadcrumb: 파일 경로 컨텍스트 유지
- gutter: 현재 line 강조
- minimap: 내용 밀도 감각 제공 (정밀 코드뷰 목적 아님)

### Status Bar

- branch/head short id, sync, save, cursor, encoding, language
- 작은 뷰포트에서는 오른쪽 상세 그룹 축약 가능

## Responsive Rule

- 모바일 전환 기준: `max-width: 720px`
- 이 이하에서 sidebar는 overlay drawer로 동작

## Mapping (Current Implementation)

- `apps/desktop/src/App.tsx`: shell structure + interactions
- `apps/desktop/src/styles/workspace.css`: component styling
- `apps/desktop/src/styles/tokens.css`: design tokens

## Prototype HTML Excerpt

전체 HTML 원문을 README에 그대로 넣으면 문서가 과도하게 길어져서 유지보수가 어렵다.  
대신 핵심만 발췌해 참조한다.

```html
<script id="tailwind-config">
tailwind.config = {
  theme: {
    extend: {
      colors: {
        "primary": "#0a78c2",
        "background-dark": "#101b22",
        "vscode-editor": "#1E1E1E",
        "vscode-sidebar": "#252526",
        "vscode-activity": "#333333",
        "vscode-text": "#D4D4D4"
      },
      fontFamily: {
        "display": ["Inter", "sans-serif"],
        "mono": ["JetBrains Mono", "monospace"]
      }
    }
  }
}
</script>
```
