use std::net::TcpListener;

use colored::Colorize;
use config;

pub fn execute() {
    // 1. dev.toml 로드
    let cfg = match config::load_dev_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("error: {}", e);
            return;
        }
    };

    // 2. services 꺼내기 + 이름순 정렬
    let mut services: Vec<_> = cfg.services.unwrap_or_default().into_iter().collect();
    services.sort_by_key(|(name, _)| name.clone());

    if services.is_empty() {
        println!("No services defined in dev.toml.");
        return;
    }

    // 3. 각 서비스 포트로 running/stopped 판단
    println!("\n{:<12} {:<10} {}", "SERVICE".dimmed(), "STATUS".dimmed(), "PORT".dimmed());
    println!("{}", "-".repeat(32).dimmed());

    for (name, svc) in services {
        // 4. 출력
        let (status, port) = match svc.port {
            Some(port) => {
                let in_use = TcpListener::bind(format!("127.0.0.1:{}", port)).is_err();
                let status = if in_use { "running".green() } else { "stopped".red() };
                (status, port.to_string())
            }
            None => ("unknown".yellow(), "-".to_string()),
        };

        println!("{:<12} {:<10} {}", name, status, port);
    }

    println!();
}
