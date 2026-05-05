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
  ✘  DATABASE_URL   `DATABASE_URL` is not set   → add DATABASE_URL to .env

  1 errors, 0 warnings
```

## 명령어

| 명령어 | 설명 |
|--------|------|
| `devctl doctor` | 개발 환경 전체 진단 |
| `devctl up` | 서비스 상태 기반 실행 (예정) |
| `devctl down` | 실행 중인 서비스 종료 (예정) |
| `devctl status` | 현재 서비스/포트 상태 확인 (예정) |
| `devctl run <task>` | `dev.toml` 태스크 실행 (예정) |
| `devctl env check` | 환경변수 스키마 검증 (예정) |

## 프로젝트 구조

Rust workspace로 구성되며, 크레이트별로 관심사가 분리됩니다.

```
devctl/
├── Cargo.toml                        # workspace 루트
├── dev.toml                          # 프로젝트 설정 (서비스, 태스크 정의)
│
└── crates/
    ├── cli/                          # 바이너리 진입점
    │   └── src/
    │       ├── main.rs               # clap 서브커맨드 라우팅
    │       ├── output.rs             # 터미널 색상/포맷 출력
    │       └── commands/
    │           └── doctor.rs         # doctor 명령어 실행 진입점
    │
    └── doctor/                       # 환경 진단 엔진 (CLI 무관)
        └── src/
            ├── lib.rs                # 공개 API 재내보내기
            ├── check.rs              # Check 트레이트, CheckResult 타입
            ├── runner.rs             # 체크 목록 실행 및 결과 수집
            └── checks/
                ├── tool.rs           # 도구 설치 여부 (git, docker 등)
                ├── port.rs           # 포트 사용 중 여부
                └── env.rs            # 환경변수 존재 여부
```

### 크레이트 의존 방향

```
cli  →  doctor
```

`doctor`는 `cli`를 모릅니다. 터미널 출력 코드가 없어 독립적으로 테스트 가능합니다.

## 설정 파일

### `dev.toml`

```toml
[project]
name = "my-api"

[services.db]
type = "docker"
image = "postgres:16"
port = 5432

[tasks.test]
cmd = "cargo test"
depends_on = ["db"]
```

### `.env.schema.toml`

```toml
[DATABASE_URL]
required = true
example = "postgres://user:pass@localhost:5432/mydb"

[PORT]
default = "3000"
```

## 개발 환경 셋업

```bash
git clone https://github.com/illus1ons/devctl
cd devctl
cargo build
cargo run -p devctl -- doctor
```

## 라이선스

MIT
