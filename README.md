# Agent Skill & MCP Inventory

로컬에 설치된 Codex, Claude Code, Gemini CLI, Antigravity의 Skill/MCP 설정을 탐색해 에이전트별·스코프별로 묶어서 보여주는 macOS 데스크톱 앱입니다.

## 핵심 기능

- `codex`, `claude`, `gemini`, `antigravity` 설정 스캔
- MCP와 SKILL 항목을 단일 모델(`DiscoveryRecord`)로 정규화
- 에이전트/스코프/위치 우선 정렬 및 검색
- 중복(이름+위치) 그룹 표시
- 에이전트별 위치 집약 뷰(어느 scope-hint 경로에 어떤 항목이 묶였는지)
- 민감 env 값 마스킹 토글
- 원문(raw)과 소스 위치 미리보기 및 파일 열기
- 수동 새로고침
- 에이전트별 지원 항목 요약(MCP / Skill / Subagent 추정치) 표시
- 캐시(파일 mtime/hash 기반) 활용: 변경되지 않은 항목은 빠르게 재사용

## 실행

```bash
pnpm install
pnpm run build
pnpm run tauri dev
```

또는

```bash
pnpm run build
pnpm run tauri build
```

### 권장 실행 진입점 (pnpm)

- 개발(브라우저): `pnpm run dev -- --host 127.0.0.1 --port 5175`
- 데스크톱 실행: `pnpm run tauri dev`
- 배포 빌드: `pnpm run deploy:mac`
- 검증 게이트: `pnpm run verify:all`
- 타입체크:
  - `pnpm run typecheck`
  - `pnpm run typecheck:full`
- 의존성 관리:
  - `pnpm run deps:check`
  - `pnpm run deps:update`
- GitHub CI 실패 점검:
  - `gh auth status` (repo/workflow scope 포함)
  - `pnpm run ci:inspect-pr -- --pr <PR_NUMBER|PR_URL> [--repo <repo-or-path>] [--json]`
  - `PR=<PR_NUMBER|PR_URL> pnpm run ci:inspect-pr`

예시:
```bash
pnpm run ci:inspect-pr -- --pr 123
PR=123 pnpm run ci:inspect-pr
pnpm run ci:inspect-pr -- --repo my-org/agent-skill-inventory --pr https://github.com/my-org/agent-skill-inventory/pull/123 --json
```

실행 시점:
- 기본적으로 현재 브랜치 PR을 자동 조회하려면 Git repo 루트에서 실행해야 합니다.
- 로컬 경로가 git root가 아니면 반드시 `--pr` 또는 `PR`로 PR을 지정해야 합니다.

### 실행 전 검증 실패 구간

`pnpm run verify:all`은 다음 순서로 실패를 분류해 보여줍니다.

- `verify:env`  
  Node/pnpm/rustc/cargo 존재 여부와 실행 가능성.
  - `react-refresh` 계열 오류가 아니라면 먼저 여기서 node/pnpm/rust가 정상인지 확인
- `verify:deps`  
  `react-refresh`, `@vitejs/plugin-react`, `@types/*`, `react/jsx-runtime`를 require.resolve로 검증
  - `Cannot find package 'react-refresh'` 같은 에러는 보통 이 구간에서 감지됨
- `verify:typescript`  
  `strict` 모드 유지와 `tsc -b` 타입 체크 통과 여부
- `verify:rust`  
  `cargo check`로 Rust API/명령 표면(tauri invoke handler) 컴파일 검증

## 스캔 규칙 요약

- Codex: `~/.codex/config.toml`, `~/.codex/skills`, `~/.codex/vendor_imports/skills`
- Claude: `~/.claude.json`, `~/.claude/settings.json`, `/Library/Application Support/ClaudeCode/managed-mcp.json`
- Claude 프로젝트 메타: `~/.claude/projects/**`, 프로젝트 루트 `/.mcp.json`, 프로젝트 스코프 `/.claude/skills`
- Gemini: `~/.gemini/settings.json`, `~/.gemini/skills`, `~/.gemini/antigravity/mcp_config.json`
- Antigravity: `~/.gemini/antigravity/mcp_config.json`, `~/.gemini/antigravity/code_tracker/**/*_mcp.json`, `~/.gemini/antigravity/skills`

기준은 현재 사용자 홈을 기준으로 하며, `~/.claude.json`의 프로젝트 경로를 추가 스캔 루트로 확장합니다.
