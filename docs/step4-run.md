# Step 4 — `devctl run <task>` 구현 가이드

## 목표 동작

```
$ devctl run test

  → migrate   [ok]   1s
  → test      [ok]   12s

$ devctl run --list

  TASK      COMMAND
  build     cargo build --release
  test      cargo test
  migrate   diesel migration run
```

---

## 사전 작업 — `dev.toml`에 tasks 추가

`dev.toml`에 tasks 섹션이 없으므로 먼저 추가합니다.

```toml
[tasks.build]
cmd = "cargo build"

[tasks.test]
cmd = "cargo test"
depends_on = ["build"]
```

---

## 사전 작업 — `TaskConfig` 구조체 추가

`crates/config/src/dev_toml.rs`에 추가합니다.

**`DevConfig`에 tasks 필드 추가:**
```rust
pub struct DevConfig {
    pub project: Option<ProjectConfig>,
    pub doctor: Option<DoctorConfig>,
    pub services: Option<HashMap<String, ServiceConfig>>,
    pub tasks: Option<HashMap<String, TaskConfig>>,   // ← 추가
}
```

**`TaskConfig` 구조체 추가:**
```rust
#[derive(Debug, Deserialize)]
pub struct TaskConfig {
    pub cmd: String,
    pub description: Option<String>,
    pub depends_on: Option<Vec<String>>,
}
```

`cmd`는 `Option`이 아닌 `String`입니다 — task에 cmd는 필수입니다.

그리고 `lib.rs`의 pub use에도 추가합니다:
```rust
pub use dev_toml::{DevConfig, DoctorConfig, ServiceConfig, TaskConfig};
```

---

## 1단계 — `main.rs`에 Run 커맨드 추가

```rust
enum Commands {
    // 기존 커맨드들...

    /// 태스크 실행
    Run {
        /// 실행할 태스크 이름
        task: Option<String>,
        /// 사용 가능한 태스크 목록 출력
        #[arg(long)]
        list: bool,
    },
}
```

`task`가 `Option<String>`인 이유: `--list` 플래그만 쓸 때는 task 이름이 없어도 되기 때문입니다.

**main 함수에 연결:**
```rust
Commands::Run { task, list } => commands::run::execute(task, list),
```

---

## 2단계 — `mod.rs`에 run 모듈 추가

```rust
pub mod run;
```

---

## 3단계 — `crates/cli/src/commands/run.rs` 생성

### 3-1. 기본 구조

```rust
use std::collections::HashSet;
use std::process::Command;
use std::time::Instant;

use colored::Colorize;
use config;

pub fn execute(task_name: Option<String>, list: bool) {
    let cfg = match config::load_dev_config() {
        Ok(cfg) => cfg,
        Err(e) => { eprintln!("  error: {}", e); return; }
    };

    let tasks = cfg.tasks.unwrap_or_default();

    if list {
        print_list(&tasks);
        return;
    }

    let name = match task_name {
        Some(n) => n,
        None => {
            eprintln!("  error: task name required. Use --list to see available tasks.");
            return;
        }
    };

    println!();
    let mut visited = HashSet::new();
    run_task(&name, &tasks, &mut visited);
    println!();
}
```

---

### 3-2. `--list` 출력

```rust
fn print_list(tasks: &std::collections::HashMap<String, config::TaskConfig>) {
    if tasks.is_empty() {
        println!("  No tasks defined in dev.toml.");
        return;
    }

    let mut keys: Vec<_> = tasks.keys().collect();
    keys.sort();

    println!();
    println!("  {:<12} {}", "TASK".dimmed(), "COMMAND".dimmed());
    println!("  {}", "-".repeat(40).dimmed());

    for key in keys {
        println!("  {:<12} {}", key, tasks[key].cmd.dimmed());
    }
    println!();
}
```

---

### 3-3. `run_task` — 핵심 로직

depends_on을 재귀적으로 먼저 실행합니다.

```rust
fn run_task(
    name: &str,
    tasks: &std::collections::HashMap<String, config::TaskConfig>,
    visited: &mut HashSet<String>,
) {
    // 순환 의존성 감지 + 중복 실행 방지
    if visited.contains(name) {
        return;
    }
    visited.insert(name.to_string());

    // 태스크 존재 여부 확인
    let task = match tasks.get(name) {
        Some(t) => t,
        None => {
            eprintln!("  error: task `{}` not found", name);
            return;
        }
    };

    // depends_on 먼저 실행 (재귀)
    for dep in task.depends_on.as_deref().unwrap_or(&[]) {
        run_task(dep, tasks, visited);
    }

    // 현재 태스크 실행
    let start = Instant::now();
    print!("  →  {:<12}", name);

    let status = Command::new("sh")
        .args(["-c", &task.cmd])
        .status();

    let elapsed = start.elapsed().as_secs();

    match status {
        Ok(s) if s.success() => println!("{}   {}s", "[ok]".green(), elapsed),
        Ok(_) => println!("{}", "[failed]".red()),
        Err(e) => {
            println!("{}", "[failed]".red());
            eprintln!("     {}", e);
        }
    }
}
```

---

## 완성 후 테스트

```bash
# 목록 확인
cargo run -p devctl -- run --list

# 단순 실행
cargo run -p devctl -- run build

# depends_on 실행 (build 먼저 실행됨)
cargo run -p devctl -- run test
```

---

## 새로 배우는 것

| 개념 | 설명 | 사용 위치 |
|------|------|----------|
| `HashSet` | 중복 없는 집합. `contains`/`insert`로 방문 여부 추적 | 순환 의존성 방지 |
| 재귀 함수 | 함수가 자기 자신을 호출 | `run_task` → `run_task` |
| `Instant` | 시간 측정 | 실행 시간 계산 |
| `as_deref()` | `Option<Vec<T>>`를 `Option<&[T]>`로 변환 | `depends_on` 처리 |

---

## 완료 기준

```
devctl run --list      → task 목록 출력
devctl run build       → cargo build 실행
devctl run test        → build 먼저 실행 후 test 실행
devctl run 없는태스크   → 에러 메시지 출력
```
