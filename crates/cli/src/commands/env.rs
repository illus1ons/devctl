use std::fs;
use std::path::Path;

use colored::Colorize;
use config;

pub fn check() {
    let env = config::load_env_file();

    let schema = match load_schema() {
        Some(s) => s,
        None => return,
    };

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
                println!("  {}  {}   {}", "⚠".yellow(), key, "set but empty".dimmed());
                warnings += 1;
            }
            Some(_) => {
                println!("  {}  {}", "✔".green(), key);
            }
            None if required => {
                println!(
                    "  {}  {}   {}   {} {}",
                    "✘".red(),
                    key,
                    "not set".dimmed(),
                    "→".dimmed(),
                    format!("add {}= to .env", key).cyan()
                );
                errors += 1;
            }
            None => {
                let msg = match &var.default {
                    Some(d) => format!("not set (default: {})", d),
                    None => "not set".to_string(),
                };
                println!("  {}  {}   {}", "⚠".yellow(), key, msg.dimmed());
                warnings += 1;
            }
        }
    }

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

pub fn diff() {
    let env = config::load_env_file();

    let schema = match load_schema() {
        Some(s) => s,
        None => return,
    };

    // 스키마에 있지만 .env에 없는 키
    let mut missing: Vec<_> = schema
        .keys()
        .filter(|k| !env.contains_key(*k))
        .collect();
    missing.sort();

    // .env에 있지만 스키마에 없는 키
    let mut extra: Vec<_> = env
        .keys()
        .filter(|k| !schema.contains_key(k.as_str()))
        .collect();
    extra.sort();

    println!();

    if missing.is_empty() && extra.is_empty() {
        println!("  {}", ".env matches schema perfectly.".green().bold());
        println!();
        return;
    }

    if !missing.is_empty() {
        println!("  {}  스키마에 있지만 .env에 없는 키:", "✘".red());
        for key in &missing {
            println!("     {}", key.red());
        }
        println!();
    }

    if !extra.is_empty() {
        println!("  {}  .env에 있지만 스키마에 없는 키:", "⚠".yellow());
        for key in &extra {
            println!("     {}", key.yellow());
        }
        println!();
    }
}

pub fn generate() {
    let schema = match load_schema() {
        Some(s) => s,
        None => return,
    };

    let output_path = ".env.example";

    if Path::new(output_path).exists() {
        println!("  {}  already exists. Overwrite? (y/N)", output_path.yellow());

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap_or(0);
        if input.trim().to_lowercase() != "y" {
            println!("  cancelled.");
            return;
        }
    }

    let mut keys: Vec<_> = schema.keys().collect();
    keys.sort();

    let mut lines = Vec::new();

    for key in keys {
        let var = &schema[key];

        if let Some(desc) = &var.description {
            lines.push(format!("# {}", desc));
        }

        let value = var
            .example
            .as_deref()
            .or(var.default.as_deref())
            .unwrap_or("");

        lines.push(format!("{}={}", key, value));
        lines.push(String::new());
    }

    let content = lines.join("\n");

    match fs::write(output_path, content) {
        Ok(_) => println!("\n  {}  {}\n", "✔".green(), output_path),
        Err(e) => eprintln!("  error: {}", e),
    }
}

fn load_schema() -> Option<config::EnvSchema> {
    match config::load_env_schema() {
        Ok(s) => Some(s),
        Err(config::ConfigError::NotFound(_)) => {
            eprintln!("  warning: .env.schema.toml not found.");
            None
        }
        Err(e) => {
            eprintln!("  error: {}", e);
            None
        }
    }
}
