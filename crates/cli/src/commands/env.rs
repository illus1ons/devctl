use colored::Colorize;
use config;

pub fn execute() {
    // 1. .env 파일과 스키마 로드
    let env = config::load_env_file();

    let schema = match config::load_env_schema() {
        Ok(s) => s,
        Err(config::ConfigError::NotFound(_)) => {
            eprintln!("  warning: .env.schema.toml not found.");
            return;
        }
        Err(e) => {
            eprintln!("  error: {}", e);
            return;
        }
    };

    // 2. 스키마 기준으로 검증
    let mut keys: Vec<_> = schema.keys().collect();
    keys.sort();

    let mut errors = 0;
    let mut warnings = 0;

    println!();

    for key in keys {
        let var = &schema[key];
        let required = var.required.unwrap_or(false);

        match env.get(key) {
            Some(val) if val.is_empty() => {
                // 키는 있지만 값이 비어있음
                println!("  {}  {}   {}", "⚠".yellow(), key, "set but empty".dimmed());
                warnings += 1;
            }
            Some(_) => {
                // 정상
                println!("  {}  {}", "✔".green(), key);
            }
            None if required => {
                // 필수인데 없음
                let fix = format!("add {}= to .env", key);
                println!(
                    "  {}  {}   {}   {} {}",
                    "✘".red(),
                    key,
                    "not set".dimmed(),
                    "→".dimmed(),
                    fix.cyan()
                );
                errors += 1;
            }
            None => {
                // 선택인데 없음 — default 있으면 표시
                let msg = match &var.default {
                    Some(default) => format!("not set (default: {})", default),
                    None => "not set".to_string(),
                };
                println!("  {}  {}   {}", "⚠".yellow(), key, msg.dimmed());
                warnings += 1;
            }
        }
    }

    // 3. 요약
    println!();
    if errors == 0 && warnings == 0 {
        println!("  {}", "All env vars OK.".green().bold());
    } else {
        println!(
            "  {} errors, {} warnings",
            errors.to_string().red().bold(),
            warnings.to_string().yellow().bold()
        );
    }
    println!();
}
