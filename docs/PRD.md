# devctl — Product Requirements Document

**버전:** 0.1  
**최종 수정:** 2026-05-05  
**상태:** Draft

---

## 1. 제품 개요

### 한 줄 정의

> 로컬 개발 환경을 자동으로 진단하고, 상태 기반으로 실행하며, 팀 전체가 동일한 환경을 재현할 수 있게 하는 CLI

### 문제 정의

개발자가 하루에 몇 번씩 마주치는 실제 고통:

| 상황 | 현재 해결 방식 | 비용 |
|------|--------------|------|
| 새 팀원 온보딩 | README 읽고 수작업 | 반나절~하루 |
| "내 로컬에선 됐는데" | Slack/GitHub Issue 핑퐁 | 30분~수시간 |
| 서비스 실행 전 의존성 확인 | 머릿속에 암기 | 실수 반복 |
| .env 키 빠짐 | 런타임 에러로 발견 | 디버깅 시간 |
| 포트 충돌 | `lsof -i :3000` 수동 실행 | 맥락 전환 비용 |

### 핵심 가치 제안

1. **진단** — 뭐가 빠졌는지 0.5초 안에 알려줌
2. **실행** — 이미 실행된 건 건드리지 않고, 빠진 것만 채움
3. **재현** — 어느 머신에서든 동일한 결과

---

## 2. 목표 & 비목표

### 목표 (Goals)

- `devctl doctor` 한 번으로 환경 이슈 전체 파악
- `devctl up` 한 번으로 서비스 올리기 (idempotent)
- `.env.schema.toml`로 환경변수 강제 계약화
- 프로젝트 루트에 `dev.toml` 하나로 팀 전체 설정 공유
- 신규 팀원이 `devctl doctor && devctl up`만으로 개발 시작 가능

### 비목표 (Non-Goals)

- CI/CD 파이프라인 대체 (GitHub Actions, ArgoCD 등과 경쟁 X)
- 원격 서버/인프라 관리
- IDE 플러그인 (CLI 완성도 우선)
- 패키지 매니저 기능 (Homebrew, npm 대체 X)
- 플러그인 마켓플레이스 (MVP 이후 검토)

---

## 3. 사용자 페르소나

### Primary: 백엔드/풀스택 개발자 (팀 단위)

- 하루 3회 이상 서비스 재시작
- DB, Redis, 메시지 큐 등 다수의 로컬 의존성 관리
- 팀원과 동일한 환경 유지가 중요

### Secondary: 새 팀원 / 프리랜서

- 프로젝트 처음 셋업하는 사람
- README 읽지 않고 바로 동작하길 원함

---

## 4. 기능 요구사항

### 우선순위 기준

- **P0**: 없으면 제품이 아님
- **P1**: MVP에 포함, 핵심 UX
- **P2**: 두 번째 릴리스
- **P3**: 이후 검토

---

### P0 — 핵심 기능

#### 4.1 `devctl doctor`

환경 상태를 진단해 문제와 수정 방법을 출력한다.

```
$ devctl doctor

[devctl] Checking environment...

  ✔  rust 1.78.0
  ✔  docker running
  ✔  .env exists
  ✔  DATABASE_URL set
  ✘  postgres not running       → devctl up db
  ✘  PORT not set in .env       → add PORT=3000
  ⚠  port 3000 already in use   → pid 58291 (node)

2 errors, 1 warning
```

**요구사항:**

- 각 Check는 독립적으로 실행 (하나 실패해도 나머지 계속)
- 에러/경고 각각 Exit code 구분 (에러: 1, 경고만: 0)
- `--fix` 플래그: 자동 수정 가능한 항목은 바로 처리
- `--json` 플래그: CI 파이프라인 연동용 JSON 출력
- 기본 내장 Check 목록:

| Check | 설명 |
|-------|------|
| `ToolCheck` | CLI 도구 설치 여부 (rust, node, docker 등) |
| `ProcessCheck` | 프로세스 실행 여부 |
| `PortCheck` | 포트 사용 가능 여부 + 어떤 프로세스가 점유 중인지 |
| `EnvCheck` | `.env.schema.toml` 기반 환경변수 존재/형식 검증 |
| `DockerServiceCheck` | docker-compose 서비스 상태 |
| `FileCheck` | 필수 파일 존재 여부 (`.env`, config 파일 등) |

---

#### 4.2 `devctl up`

`dev.toml`에 선언된 서비스를 상태 기반으로 실행한다.

```
$ devctl up

  → db       already running   [skip]
  → redis    starting...       [ok]
  → app      starting...       [ok]

All services up.
```

**요구사항:**

- 실행 전 자동으로 `doctor` 의존성 체크 수행
- 이미 실행 중인 서비스는 건드리지 않음 (idempotent)
- `devctl up <service>` — 특정 서비스만 실행
- `devctl up --no-check` — doctor 체크 건너뜀
- 서비스 타입별 실행 전략:
  - `docker` — `docker run` 또는 `docker-compose up`
  - `process` — 직접 명령어 실행 (백그라운드)
  - `command` — 한 번만 실행하는 셋업 명령어

---

#### 4.3 `.env.schema.toml` — 환경변수 계약

프로젝트에 필요한 환경변수를 명세로 선언한다.

```toml
[DATABASE_URL]
required = true
description = "PostgreSQL connection string"
example = "postgres://user:pass@localhost:5432/mydb"

[PORT]
required = false
default = "3000"
type = "number"

[SECRET_KEY]
required = true
secret = true          # 값 검증만 하고 출력 시 마스킹
min_length = 32

[APP_ENV]
required = true
allowed = ["development", "staging", "production"]
```

**요구사항:**

- `devctl env check` — 스키마 대비 현재 `.env` 검증
- `devctl env diff` — 현재 `.env`와 스키마 비교 (팀원 간 누락 키 파악)
- `devctl env generate` — 스키마에서 `.env.example` 자동 생성
- 지원 타입: `string`(기본), `number`, `boolean`, `url`
- 검증 실패 시 누락/형식 오류 항목과 `example` 값을 함께 출력

---

#### 4.4 `dev.toml` — 프로젝트 설정

```toml
[project]
name = "my-api"
description = "Backend API server"

[services.db]
type = "docker"
image = "postgres:16"
port = 5432
env = { POSTGRES_PASSWORD = "dev" }
health_check = "pg_isready -U postgres"

[services.redis]
type = "docker"
image = "redis:7"
port = 6379

[services.app]
type = "process"
cmd = "cargo run"
port = 3000
depends_on = ["db", "redis"]
watch = ["src/**/*.rs"]   # 파일 변경 시 재시작

[tasks.build]
cmd = "cargo build --release"

[tasks.test]
cmd = "cargo test"
depends_on = ["db"]

[tasks.migrate]
cmd = "diesel migration run"
depends_on = ["db"]
```

---

### P1 — MVP 포함, 중요 기능

#### 4.5 `devctl run <task>`

`dev.toml`에 정의된 task를 의존성 순서대로 실행한다.

```
$ devctl run test

  → migrate  [running]
  → test     [running]

  test result: ok. 42 passed; 0 failed
```

**요구사항:**

- 의존성 그래프(DAG) 기반 실행 순서 결정
- 순환 의존성 감지 시 에러 출력
- `devctl run --list` — 사용 가능한 task 목록
- task 실행 전 관련 서비스가 `up` 상태인지 체크
- 실패한 task는 즉시 중단 + 원인 출력

---

#### 4.6 `devctl status`

현재 환경의 모든 서비스/포트 상태를 한눈에 표시한다.

```
$ devctl status

SERVICE   TYPE     STATUS    PORT   UPTIME
db        docker   running   5432   2h 14m
redis     docker   running   6379   2h 14m
app       process  stopped   3000   —

PORTS IN USE (not in dev.toml)
  8080  node (pid 12391)
  4200  ng (pid 98712)
```

**요구사항:**

- `dev.toml` 서비스 상태 + 현재 시스템에서 사용 중인 포트 표시
- 정의된 포트와 충돌하는 외부 프로세스 경고
- `devctl status --watch` — 터미널에서 실시간 갱신 (1초 간격)

---

#### 4.7 `devctl down`

실행 중인 서비스를 정리한다.

```
$ devctl down

  → app   stopping...  [ok]
  → redis stopping...  [ok]
  → db    stopping...  [ok]
```

**요구사항:**

- `devctl down <service>` — 특정 서비스만 중지
- `devctl down --all` — `dev.toml` 외 devctl이 시작한 모든 프로세스 정리
- 의존성 역순으로 종료 (app → redis → db)

---

#### 4.8 `devctl init`

기존 프로젝트를 분석해 `dev.toml`과 `.env.schema.toml`을 자동 생성한다.

```
$ devctl init

  Detected: Rust (Cargo.toml)
  Detected: docker-compose.yml (postgres, redis)
  Detected: .env.example

  Generating dev.toml...        [ok]
  Generating .env.schema.toml...  [ok]

  Next: review dev.toml and run `devctl doctor`
```

**감지 대상:**

| 파일/패턴 | 추론 결과 |
|----------|----------|
| `Cargo.toml` | Rust 프로젝트, `cargo build/run/test` task 생성 |
| `package.json` | Node 프로젝트, scripts를 task로 변환 |
| `docker-compose.yml` | 서비스 설정 자동 임포트 |
| `.env.example` | `.env.schema.toml` 자동 생성 |
| `Makefile` | make target을 task로 변환 |
| `Procfile` | 서비스 정의로 변환 |

---

### P2 — 두 번째 릴리스

#### 4.9 `devctl history` / `devctl replay`

실행한 커맨드와 결과를 기록하고 재현한다.

```
$ devctl history

  #  COMMAND           STATUS   TIME        DURATION
  5  run test          ok       14:22:01    12s
  4  up                ok       14:20:44    3s
  3  env check         error    14:18:02    0s
  2  run migrate       ok       09:11:33    1s
  1  up db             ok       09:11:28    2s

$ devctl replay 4
```

**저장 정보:** command, exit code, stdout/stderr, 실행 시간, duration, 실행 디렉토리

**저장소:** `~/.devctl/history.db` (SQLite)

---

#### 4.10 `devctl doctor --profile <name>`

팀별/역할별 진단 프로파일을 지원한다.

```toml
# dev.toml
[doctor.profiles.backend]
checks = ["rust", "postgres", "redis", "env"]

[doctor.profiles.frontend]
checks = ["node", "env"]
```

```
$ devctl doctor --profile backend
```

---

#### 4.11 커스텀 Check 지원

`dev.toml`에서 직접 체크 조건 정의:

```toml
[[doctor.checks]]
name = "migrations up to date"
cmd = "diesel migration pending"
expect_exit = 0
on_fail = "run: devctl run migrate"

[[doctor.checks]]
name = "api reachable"
url = "http://localhost:3000/health"
expect_status = 200
```

---

### P3 — 이후 검토

- 플러그인 시스템 (외부 Check/Task 로드)
- `devctl share` — 팀원에게 현재 환경 상태 스냅샷 공유
- `devctl doctor --ci` — GitHub Actions 등 CI 환경 특화 체크
- Shell completion (zsh, bash, fish)
- `devctl update` — 자기 자신 업데이트

---

## 5. 기술 아키텍처

### 언어 & 핵심 의존성

| 용도 | 크레이트 |
|------|---------|
| CLI 파싱 | `clap` (derive) |
| 설정 파싱 | `serde` + `toml` |
| 비동기 런타임 | `tokio` |
| DB (history) | `rusqlite` |
| 터미널 출력 | `console` + `indicatif` |
| 프로세스 관리 | `std::process` + `nix` |
| HTTP (health check) | `reqwest` |

### Workspace 구조

```
devctl/
├── Cargo.toml               # workspace
├── crates/
│   ├── cli/                 # clap 엔트리포인트, 명령어 라우팅
│   ├── core/                # 공통 타입, 에러, 트레이트
│   ├── config/              # dev.toml / .env.schema.toml 로딩
│   ├── doctor/              # Check 트레이트 + 내장 구현체들
│   ├── services/            # 서비스 상태 관리 (up/down/status)
│   ├── tasks/               # DAG 기반 task 실행 엔진
│   ├── env/                 # 환경변수 스키마 검증
│   └── history/             # SQLite 기반 실행 이력
└── dev.toml                 # devctl 자체 설정 (dogfooding)
```

### 핵심 트레이트 설계

```rust
// doctor/src/lib.rs
pub enum CheckStatus {
    Ok,
    Warning(String),
    Error { message: String, fix: Option<String> },
}

pub trait Check: Send + Sync {
    fn name(&self) -> &str;
    fn run(&self) -> CheckStatus;
}

// services/src/lib.rs
pub enum ServiceState {
    Running { pid: Option<u32>, uptime: Duration },
    Stopped,
    Unknown,
}

pub trait ServiceDriver: Send + Sync {
    fn state(&self, svc: &ServiceConfig) -> ServiceState;
    fn start(&self, svc: &ServiceConfig) -> Result<()>;
    fn stop(&self, svc: &ServiceConfig) -> Result<()>;
}
```

### 상태 파일

```
~/.devctl/
├── history.db          # SQLite 실행 이력
└── state/
    └── <project-hash>/ # 프로젝트별 서비스 상태 캐시
        └── services.json
```

---

## 6. CLI 인터페이스 명세

```
devctl <COMMAND>

COMMANDS:
  doctor    환경 진단
  up        서비스 실행
  down      서비스 종료
  status    현재 상태 확인
  run       태스크 실행
  env       환경변수 관리
  history   실행 이력
  replay    이전 명령 재실행
  init      프로젝트 초기화

FLAGS (공통):
  --config <path>    dev.toml 경로 지정 (기본: 상위 디렉토리 탐색)
  --no-color         색상 출력 비활성화
  --json             JSON 출력 (스크립트 연동)
  -q, --quiet        에러만 출력
  -v, --verbose      상세 출력
```

### 종료 코드

| 코드 | 의미 |
|------|------|
| 0 | 성공 (경고 포함) |
| 1 | 에러 발생 |
| 2 | 설정 파일 오류 |
| 3 | 의존성 누락 |

---

## 7. UX 원칙

### 출력 원칙

1. **첫 줄에 결론** — 문제가 있으면 맨 위에 요약
2. **수정 방법 제시** — 에러 출력 시 항상 다음 액션 포함
3. **침묵하지 않음** — 성공해도 무엇을 했는지 1줄 이상 출력
4. **skip은 명시적으로** — 아무것도 안 한 것과 skip을 구분

### 설정 탐색 순서

1. `--config` 플래그
2. 현재 디렉토리 `dev.toml`
3. 상위 디렉토리 순차 탐색 (git root까지)
4. 없으면 에러 + `devctl init` 안내

---

## 8. 성공 지표

### 정성 지표

- 신규 팀원이 `devctl doctor && devctl up`만으로 첫 실행 성공
- `devctl doctor`가 README 없이 다음 액션을 알려줌
- 기존 `make dev` / `docker-compose up` 스크립트를 대체

### 정량 지표 (MVP 이후 측정)

- `devctl doctor` 실행 시간 < 1초
- `devctl up` (서비스 이미 실행 중) 실행 시간 < 0.5초
- GitHub Star 100개 (첫 3개월)

---

## 9. 개발 로드맵

### Phase 1 — Foundation (1주)

- [ ] Workspace 구조 셋업
- [ ] `config` 크레이트: `dev.toml` + `.env.schema.toml` 파싱
- [ ] `doctor` 크레이트: `ToolCheck`, `PortCheck`, `EnvCheck`
- [ ] `cli` 크레이트: `devctl doctor` 명령어

**완료 기준:** `devctl doctor`가 포트/env/도구 체크를 실행하고 수정 방법을 출력

### Phase 2 — Run (1주)

- [ ] `services` 크레이트: Docker/Process 드라이버
- [ ] `devctl up` / `devctl down` / `devctl status`
- [ ] `tasks` 크레이트: DAG 실행 엔진
- [ ] `devctl run <task>`

**완료 기준:** `dev.toml`로 서비스 올리고 task 실행 가능

### Phase 3 — Polish (3일)

- [ ] `devctl init` (docker-compose, package.json 감지)
- [ ] `devctl env check/diff/generate`
- [ ] `--json` 플래그
- [ ] Shell completion

**완료 기준:** 실제 프로젝트에서 dogfooding 가능

### Phase 4 — History (3일)

- [ ] `history` 크레이트: SQLite 저장
- [ ] `devctl history` / `devctl replay`

---

## 10. 리스크 & 대응

| 리스크 | 가능성 | 대응 |
|--------|--------|------|
| Docker 미설치 환경에서 docker check 실패 | 높음 | Docker check는 docker가 있을 때만 활성화 |
| dev.toml 없는 프로젝트에서 실행 | 높음 | 친절한 에러 + `devctl init` 안내 |
| 포트 감지 OS별 동작 차이 | 중간 | `ss`/`lsof`/`netstat` fallback 체인 |
| 장시간 실행 서비스 상태 추적 | 중간 | PID 파일 기반 상태 캐시 |
