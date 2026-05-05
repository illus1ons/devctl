# devctl

> 로컬 개발 환경을 자동으로 진단하고, 상태 기반으로 실행하며, 팀 전체가 동일한 환경을 재현할 수 있는 CLI

## 빠른 시작

```bash
cargo install devctl
devctl doctor
```

```
[devctl] Checking environment...

  ✔  git
  ✔  docker
  ✔  db (port 5432)
  ✘  DATABASE_URL   `DATABASE_URL` is not set   → add DATABASE_URL to .env

  1 errors, 0 warnings
```

---

## 명령어

### `devctl doctor`

`dev.toml` 기반으로 개발 환경 전체를 진단한다.

```bash
devctl doctor
```

- `[doctor.tools]` — CLI 도구 설치 여부 확인
- `[services.*]` — 포트 기반 서비스 실행 여부 확인
- `.env.schema.toml` — 필수 환경변수 누락 여부 확인
- 문제 발견 시 수정 방법(`→`)을 함께 출력

---

### `devctl status`

현재 서비스 실행 상태를 한눈에 확인한다.

```bash
devctl status
```

```
SERVICE      STATUS     PORT
--------------------------------
db           stopped    5432
redis        running    6379
```

---

### `devctl up`

`dev.toml`의 서비스를 상태 기반으로 실행한다. 이미 실행 중인 서비스는 건드리지 않는다.

```bash
devctl up           # 전체 서비스 실행
devctl up db        # 특정 서비스만 실행
```

```
  →  db       already running [skip]
  →  redis    starting...  [ok]
```

- `type = "docker"` — `docker run` 으로 백그라운드 실행
- `type = "process"` — `sh -c <cmd>` 로 실행
- 컨테이너 이름 규칙: `devctl-<project>-<service>`

---

### `devctl down`

실행 중인 서비스를 종료한다. `up`의 역순으로 종료한다.

```bash
devctl down         # 전체 서비스 종료
devctl down redis   # 특정 서비스만 종료
```

```
  →  redis    stopping...  [ok]
  →  db       stopping...  [ok]
```

---

### `devctl run <task>`

`dev.toml`의 `[tasks.*]`에 정의된 태스크를 실행한다. `depends_on`에 선언된 태스크를 먼저 실행한다.

```bash
devctl run build        # 단일 태스크 실행
devctl run test         # depends_on 순서대로 실행
devctl run --list       # 사용 가능한 태스크 목록
```

```
  →  migrate   [ok]   1s
  →  test      [ok]   12s
```

**`dev.toml` 태스크 정의:**

```toml
[tasks.build]
cmd = "cargo build"
description = "프로젝트 빌드"

[tasks.test]
cmd = "cargo test"
description = "테스트 실행"
depends_on = ["build"]

[tasks.migrate]
cmd = "diesel migration run"
depends_on = ["db"]
```

**`--list` 출력:**
```
  TASK      COMMAND
  ----------------------------------------
  build     cargo build
  migrate   diesel migration run
  test      cargo test
```

**의존성 실행 규칙:**
- `depends_on`에 선언된 태스크를 재귀적으로 먼저 실행
- 이미 실행한 태스크는 중복 실행하지 않음
- 순환 의존성(`A → B → A`) 감지 시 중단

---

### `devctl env`

`.env.schema.toml` 기반으로 환경변수를 관리한다.

```bash
devctl env check      # .env 검증
devctl env diff       # 스키마와 .env 차이 비교
devctl env generate   # .env.example 자동 생성
```

**`env check`:**
```
  ✔  DATABASE_URL
  ✘  SECRET_KEY   not set   → add SECRET_KEY= to .env
  ⚠  PORT         not set (default: 3000)
```

**`env diff`:**
```
  ✘  스키마에 있지만 .env에 없는 키:
     SECRET_KEY

  ⚠  .env에 있지만 스키마에 없는 키:
     OLD_API_KEY
```

**`env generate`** — 스키마의 `description`, `example`, `default`를 읽어 `.env.example` 생성

---

## 설정 파일

### `dev.toml`

프로젝트의 서비스, 태스크, 진단 항목을 선언한다. 팀 전체가 공유하는 파일이다.

```toml
#:schema https://raw.githubusercontent.com/illus1ons/devctl/main/schemas/dev.schema.json

[project]
name = "my-api"

[doctor]
tools = ["git", "docker", "cargo"]

[services.db]
type = "docker"
image = "postgres:16"
port = 5432

[services.redis]
type = "docker"
image = "redis:7"
port = 6379

[tasks.build]
cmd = "cargo build"

[tasks.test]
cmd = "cargo test"
depends_on = ["build"]

[tasks.migrate]
cmd = "diesel migration run"
depends_on = ["db"]
```

> VS Code에 [Even Better TOML](https://marketplace.visualstudio.com/items?itemName=tamasfe.even-better-toml) 확장을 설치하면 `#:schema` 주석을 읽어 자동완성이 동작한다.

### `.env.schema.toml`

프로젝트에 필요한 환경변수의 명세를 선언한다. `.env`는 git에서 제외하고, 이 파일은 팀 전체가 공유한다.

```toml
[DATABASE_URL]
required = true
description = "PostgreSQL 연결 문자열"
example = "postgres://user:pass@localhost:5432/mydb"

[PORT]
required = false
default = "3000"

[SECRET_KEY]
required = true
```

---

## 프로젝트 구조

```
devctl/
├── Cargo.toml                         # workspace 루트
├── dev.toml                           # 프로젝트 설정
├── .env.schema.toml                   # 환경변수 명세
├── schemas/
│   └── dev.schema.json                # dev.toml IDE 자동완성용 JSON Schema
│
└── crates/
    ├── cli/                           # 바이너리 진입점
    │   └── src/
    │       ├── main.rs                # clap 서브커맨드 라우팅
    │       ├── output.rs              # 터미널 색상/포맷 출력
    │       └── commands/
    │           ├── doctor.rs          # 환경 진단
    │           ├── status.rs          # 서비스 상태 확인
    │           ├── up.rs              # 서비스 실행
    │           ├── down.rs            # 서비스 종료
    │           ├── run.rs             # 태스크 실행 (개발 중)
    │           └── env.rs             # 환경변수 관리
    │
    ├── doctor/                        # 환경 진단 엔진 (CLI 무관)
    │   └── src/
    │       ├── check.rs               # Check 트레이트, CheckResult 타입
    │       ├── runner.rs              # 체크 목록 실행 및 결과 수집
    │       └── checks/
    │           ├── tool.rs            # 도구 설치 여부
    │           ├── port.rs            # 포트 사용 중 여부
    │           └── env.rs             # 환경변수 존재 여부
    │
    └── config/                        # 설정 파일 파싱
        └── src/
            ├── dev_toml.rs            # dev.toml 구조체
            └── env_schema.rs          # .env.schema.toml 구조체
```

### 크레이트 의존 방향

```
cli  →  doctor
cli  →  config
```

`doctor`와 `config`는 `cli`를 모릅니다. 터미널 출력 코드 없이 독립적으로 테스트 가능합니다.

---

## 개발 환경 셋업

```bash
git clone https://github.com/illus1ons/devctl
cd devctl
cargo build
cargo run -p devctl -- doctor
```

## 라이선스

MIT
