# devctl 개발 계획

> 이 문서를 순서대로 따라가면 됩니다.  
> 각 단계는 독립적으로 커밋 가능한 단위입니다.

---

## 현재 완료된 것

- [x] workspace 구조 (cli / doctor / config)
- [x] `devctl doctor` — dev.toml 기반 환경 진단
- [x] `devctl status` — 서비스 실행 상태 확인
- [x] `dev.toml` JSON Schema

---

## Step 1 — `devctl env` (환경변수 관리)

**목표:** `.env.schema.toml` 기반으로 환경변수를 검증하고 관리한다.

### 1-1. `devctl env check`

`.env` 파일을 읽어 스키마와 대조한다.

```
$ devctl env check

  ✔  DATABASE_URL
  ✘  SECRET_KEY   not set   → add SECRET_KEY to .env
  ⚠  PORT         not set   (default: 3000)

  1 errors, 1 warnings
```

**구현할 것:**
- `crates/cli/src/commands/env.rs` 생성
- `.env` 파일 파싱 (표준 라이브러리로 직접 구현 — `KEY=VALUE` 형식)
- `config::load_env_schema()` 로 스키마 로드
- 스키마의 `required = true` 항목이 `.env`에 없으면 에러
- `default` 가 있는 항목이 없으면 경고

**새로 배우는 것:**
- 파일 읽기: `std::fs::read_to_string`
- 문자열 파싱: `lines()`, `split_once('=')`

---

### 1-2. `devctl env diff`

팀원과 `.env` 키 차이를 비교한다.

```
$ devctl env diff

  스키마 기준 누락된 키:
  ✘  SECRET_KEY
  ✘  REDIS_URL

  .env에는 있지만 스키마에 없는 키:
  ⚠  OLD_API_KEY   (스키마에 정의되지 않음)
```

**구현할 것:**
- `env check` 확장
- 스키마에 없는 키도 역방향으로 체크

---

### 1-3. `devctl env generate`

스키마에서 `.env.example` 자동 생성

```
$ devctl env generate

  Generated .env.example
```

`.env.example` 내용:
```
# PostgreSQL 연결 문자열
DATABASE_URL=postgres://user:pass@localhost:5432/mydb

# 기본값: 3000
PORT=3000
```

**구현할 것:**
- 스키마의 `description`, `example`, `default` 를 읽어 파일 작성
- 이미 `.env.example` 이 있으면 덮어쓸지 확인

---

**완료 기준:**
```bash
devctl env check    # 에러/경고 출력
devctl env diff     # 스키마 vs .env 비교
devctl env generate # .env.example 생성
```

---

## Step 2 — `devctl up` (서비스 실행)

**목표:** `dev.toml`의 서비스를 상태 기반으로 실행한다. 이미 실행 중이면 건드리지 않는다.

```
$ devctl up

  → db     already running   [skip]
  → redis  starting...       [ok]
```

### 2-1. Docker 서비스 실행

**구현할 것:**
- `crates/cli/src/commands/up.rs` 생성
- `std::process::Command` 로 `docker run` 실행
- 실행 전 포트 체크 → 이미 사용 중이면 skip
- `--detach` 플래그로 백그라운드 실행
- 컨테이너 이름 규칙: `devctl-<project>-<service>`

```rust
// docker run 예시
Command::new("docker")
    .args(["run", "--detach",
           "--name", &container_name,
           "-p", &format!("{}:{}", port, port)])
    .arg(&image)
    .status()
```

**새로 배우는 것:**
- `std::process::Command` 로 외부 명령어 실행
- `.status()` vs `.output()` 차이
  - `status()` — exit code만 필요할 때
  - `output()` — stdout/stderr 내용이 필요할 때

---

### 2-2. `devctl up <service>`

특정 서비스만 실행

```bash
devctl up db
```

**구현할 것:**
- CLI에 optional argument 추가

```rust
#[derive(Subcommand)]
enum Commands {
    Up { service: Option<String> },
}
```

---

**완료 기준:**
```bash
devctl up          # 전체 서비스 실행
devctl up db       # db만 실행
devctl up          # 다시 실행해도 이미 running이면 skip
```

---

## Step 3 — `devctl down` (서비스 종료)

**목표:** 실행 중인 서비스를 정리한다.

```
$ devctl down

  → redis  stopping...  [ok]
  → db     stopping...  [ok]
```

**구현할 것:**
- `crates/cli/src/commands/down.rs` 생성
- `docker stop <container_name>` 실행
- `docker rm <container_name>` 실행
- 의존성 역순으로 종료 (depends_on 역방향)
- `devctl down <service>` — 특정 서비스만 종료

**새로 배우는 것:**
- `Vec::reverse()` 로 순서 뒤집기
- `depends_on` 기반 위상 정렬 (간단한 버전)

---

**완료 기준:**
```bash
devctl down        # 전체 종료
devctl down redis  # redis만 종료
devctl up          # 다시 올리면 정상 동작
```

---

## Step 4 — `devctl run <task>` (태스크 실행)

**목표:** `dev.toml`의 task를 의존성 순서대로 실행한다.

```
$ devctl run test

  → migrate  [ok]  1s
  → test     [ok]  12s
```

### 4-1. 단순 task 실행

**구현할 것:**
- `crates/cli/src/commands/run.rs` 생성
- `dev.toml`의 `[tasks.*]` 읽기
- `std::process::Command` 로 `cmd` 실행
- 실행 시간 측정: `std::time::Instant`

```rust
let start = Instant::now();
let status = Command::new("sh").args(["-c", &task.cmd]).status()?;
let elapsed = start.elapsed();
```

---

### 4-2. depends_on 처리

task의 `depends_on` 순서대로 먼저 실행

**구현할 것:**
- 재귀적으로 의존성 먼저 실행
- 순환 의존성 감지 (A → B → A 이면 에러)

**새로 배우는 것:**
- 재귀 함수
- `HashSet` 으로 방문 여부 추적

---

### 4-3. `devctl run --list`

사용 가능한 task 목록 출력

```
$ devctl run --list

  build    cargo build --release
  test     cargo test
  migrate  diesel migration run
```

---

**완료 기준:**
```bash
devctl run build         # 단순 실행
devctl run test          # depends_on: [db] 먼저 확인
devctl run --list        # task 목록
```

---

## Step 5 — `devctl init` (프로젝트 초기화)

**목표:** 기존 프로젝트를 분석해서 `dev.toml`과 `.env.schema.toml`을 자동 생성한다.

```
$ devctl init

  Detected: Cargo.toml       → rust 프로젝트
  Detected: docker-compose.yml → db, redis 서비스
  Detected: .env.example     → 환경변수 3개

  Generated dev.toml
  Generated .env.schema.toml
```

**감지 대상 & 처리:**

| 파일 | 동작 |
|------|------|
| `Cargo.toml` | `tools = ["cargo"]`, `tasks.build/test` 추가 |
| `package.json` | `tools = ["node"]`, scripts → tasks 변환 |
| `docker-compose.yml` | services 파싱해서 그대로 임포트 |
| `.env.example` | 키 추출해서 `.env.schema.toml` 생성 |
| `Makefile` | target 추출해서 tasks 생성 |

**구현할 것:**
- `crates/cli/src/commands/init.rs` 생성
- 각 파일 존재 여부 체크
- `docker-compose.yml` 파싱: `serde_yaml` 크레이트 추가
- 이미 `dev.toml` 이 있으면 덮어쓸지 확인

**새로 배우는 것:**
- `Path::exists()` 로 파일 존재 확인
- `serde_yaml` 로 YAML 파싱

---

**완료 기준:**
```bash
# docker-compose.yml 있는 프로젝트에서
devctl init
cat dev.toml      # 서비스가 자동으로 들어가 있어야 함
```

---

## Step 6 — `devctl history` / `devctl replay` (실행 이력)

**목표:** 실행한 명령어를 SQLite에 저장하고 재현한다.

```
$ devctl history

  #  COMMAND       STATUS   TIME       DURATION
  3  run test      ok       14:22:01   12s
  2  up            ok       14:20:44   3s
  1  env check     error    14:18:02   0s

$ devctl replay 2
```

**구현할 것:**
- `crates/history/` 크레이트 생성
- `rusqlite` 크레이트로 SQLite 연동
- `~/.devctl/history.db` 에 저장
- 모든 커맨드 실행 전후로 기록 (main.rs에서 hook)

```sql
CREATE TABLE history (
    id      INTEGER PRIMARY KEY,
    command TEXT NOT NULL,
    status  TEXT NOT NULL,
    dir     TEXT NOT NULL,
    started_at DATETIME NOT NULL,
    duration_ms INTEGER NOT NULL
);
```

**새로 배우는 것:**
- SQLite with `rusqlite`
- `dirs` 크레이트로 홈 디렉토리 경로 얻기

---

**완료 기준:**
```bash
devctl up
devctl run test
devctl history     # 두 줄 출력
devctl replay 1    # up 재실행
```

---

## 크레이트 추가 시점 요약

| 단계 | 추가할 크레이트 | 이유 |
|------|--------------|------|
| Step 1 | 없음 | 기존 config 크레이트 확장 |
| Step 2~3 | 없음 | std::process::Command로 충분 |
| Step 4 | 없음 | 기존 구조 확장 |
| Step 5 | `serde_yaml` | docker-compose.yml 파싱 |
| Step 6 | `rusqlite`, `dirs` | SQLite, 홈 디렉토리 |

---

## 전체 진행 상황

```
[x] doctor
[x] status
[ ] env check/diff/generate   ← 다음
[ ] up
[ ] down
[ ] run
[ ] init
[ ] history / replay
```
