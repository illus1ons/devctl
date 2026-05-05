use std::process::Command;

use colored::Colorize;
use config;

pub fn execute(service_filter: Option<String>) {
    let cfg = match config::load_dev_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("  error: {}", e);
            return;
        }
    };

    let project_name = cfg
        .project
        .as_ref()
        .map(|p| p.name.as_str())
        .unwrap_or("devctl");

    let mut services: Vec<_> = cfg.services.unwrap_or_default().into_iter().collect();

    // up의 역순으로 종료
    services.sort_by_key(|(name, _)| name.clone());
    services.reverse();

    if services.is_empty() {
        println!("  No services defined in dev.toml.");
        return;
    }

    println!();

    for (name, svc) in &services {
        if let Some(ref filter) = service_filter {
            if name != filter {
                continue;
            }
        }

        let service_type = svc.service_type.as_deref().unwrap_or("docker");

        match service_type {
            "docker" => stop_docker(name, project_name),
            "process" => {
                println!("  →  {:<12} {}", name, "process stop not supported yet".dimmed());
            }
            unknown => {
                eprintln!("  error: unknown service type `{}`", unknown);
            }
        }
    }

    println!();
}

fn stop_docker(name: &str, project: &str) {
    let container_name = format!("devctl-{}-{}", project, name);

    print!("  →  {:<12} stopping...  ", name);

    // docker stop
    let stop = Command::new("docker")
        .args(["stop", &container_name])
        .output();

    match stop {
        Ok(o) if o.status.success() => {}
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            if stderr.contains("No such container") {
                println!("{}", "[not running]".dimmed());
                return;
            }
            println!("{}", "[failed]".red());
            eprintln!("     {}", stderr.trim().dimmed());
            return;
        }
        Err(e) => {
            println!("{}", "[failed]".red());
            eprintln!("     {}", e);
            return;
        }
    }

    // docker rm
    let _ = Command::new("docker")
        .args(["rm", &container_name])
        .output();

    println!("{}", "[ok]".green());
}
