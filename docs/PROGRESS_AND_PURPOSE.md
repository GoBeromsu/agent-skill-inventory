# Agent Skill & MCP Inventory - 진행상황 및 목적

기준 문서: 현재까지 구현된 기능과 운영 상태를 한 번에 공유하기 위한 내부 상태 문서  
기준일: 2026-02-21

## 1) 우리가 이 앱을 만드는 목적

이 앱의 핵심 목적은 아래 2가지를 해결하는 것입니다.

- 에이전트(Codex / Claude Code / Gemini CLI / Antigravity)별로 흩어진 Skill/MCP 설정을 한 화면에서 통합 조회
- 스코프(`global`, `personal`, `project`, `managed`, `system`, `session`)와 위치(파일 경로) 기준으로 구조를 파악하고 중복/유사 항목을 빠르게 탐색

사용자 페인포인트 관점에서의 목표는 다음과 같습니다.

- 내가 가진 스킬/설정이 어디에 있는지 모름
- 어떤 에이전트가 어떤 개념(MCP, Skill, Subagent)을 실제로 쓰는지 파악 어려움
- 여러 에이전트를 한 번에 운영/업데이트하는 관리 레이어(대시보드) 필요

## 2) 현재까지 구현 완료된 내용

### A. 제품/도메인 기능

- 4개 에이전트 스캔 로직 구현:
  - `codex`, `claude`, `gemini`, `antigravity`
- 통합 정규화 모델(`DiscoveryRecord`) 기반으로 Skill/MCP 데이터 수집 및 표시
- 기본 정렬/필터:
  - `AGENT > SCOPE > LOCATION > NAME`
  - 검색(이름/경로/설명), 상태 필터, 종류 필터
- 중복 표시:
  - `canonical_group_id` 기반으로 중복 그룹 추적
- 상세 정보:
  - raw 표시, source 경로 표시, 민감 env 값 마스킹/토글
- 운영 액션:
  - 현재 에이전트 기준 새로고침
  - 전체 에이전트 새로고침
  - 경로 열기(`open_path`)

### B. UI/탐색 구조

- 그리드(갤러리) 중심 탐색
- 계층 시각화 탭(에이전트/스코프/위치 트리)로 depth 기반 탐색 가능
- 에이전트별 지원 개념 요약(MCP / Skill / Subagent) 표시

### C. 개발/운영 안정성

- `pnpm` 중심 워크플로우로 통일
- 실행 전 검증 게이트 추가:
  - `verify:env`
  - `verify:deps`
  - `verify:typescript`
  - `verify:rust`
  - `verify:all`
- `react-refresh`/`@types/*` 누락 같은 반복 오류를 조기 탐지하도록 개선
- Tauri 공개 command 표면 정리(중복 command 제거 및 입력 검증 강화)
- CI 파이프라인에 검증 단계를 명시적으로 분리:
  - env/deps/typescript/rust/build/tauri build

### D. CI 진단 확장 (gh-fix-ci 연계)

- PR 실패 체크 진단 스크립트 진입점 추가:
  - `pnpm run ci:inspect-pr -- --pr <PR_NUMBER|PR_URL>`
  - `PR=<PR_NUMBER|PR_URL> pnpm run ci:inspect-pr`
- 로컬이 git root가 아닐 경우 PR 자동 해석 대신 명시 입력으로 동작하도록 구성

## 3) 현재 상태 요약

- 개발 실행(`tauri dev`)은 가능
- 프론트엔드 dev 서버 포트는 `127.0.0.1:5175` 기준으로 정렬
- `verify:all` 기준으로 환경/의존성/타입/러스트 사전 점검 가능
- 릴리스 번들 중 DMG 단계(`bundle_dmg.sh`)는 별도 안정화가 필요

## 4) 아직 남은 일 (우선순위)

1. 배포 단계 안정화
- DMG 번들 실패 원인 분리 및 macOS 배포 파이프 정리
- 필요 시 notarization/서명 전략 정리

2. Git 운영 경계 정리
- 현재 작업 경로를 명확한 git 루트/원격과 연결
- 브랜치/PR 기준 자동 진단(`gh-fix-ci`)을 완전 자동화

3. 스캔 규칙 확장
- 에이전트별 추가 managed/plugin/session 케이스 탐지 강화
- 사용자 지정 경로 목록 기반 확장 스캔 옵션 제공

4. 대시보드 고도화
- 다중 에이전트 일괄 업데이트/작업 실행 인터페이스
- 계층 시각화에서 흐름 추적(예: 시스템 문서 계층) 강화

## 5) 팀 운영 규약 (현재 권장)

- 개발 시작:
  - `pnpm install`
  - `pnpm run verify:all`
  - `pnpm run tauri dev`
- PR 체크 분석:
  - `pnpm run ci:inspect-pr -- --pr <PR>`
- 배포 시도:
  - `pnpm run deploy:mac`
  - 실패 시 번들 단계 로그를 분리해 원인 분류

