use std::net::TcpListener;
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
    services.sort_by_key(|(name, _)| name.clone());

    if services.is_empty() {
        println!("  No services defined in dev.toml.");
        return;
    }

    println!();

    for (name, svc) in &services {
        // 특정 서비스만 실행하는 경우 필터링
        if let Some(ref filter) = service_filter {
            if name != filter {
                continue;
            }
        }

        let port_in_use = svc.port.map(|port| {
            TcpListener::bind(format!("127.0.0.1:{}", port)).is_err()
        });

        // 이미 실행 중이면 skip
        if port_in_use == Some(true) {
            println!("  →  {:<12} {}", name, "already running [skip]".dimmed());
            continue;
        }

        let service_type = svc.service_type.as_deref().unwrap_or("docker");

        match service_type {
            "docker" => start_docker(name, project_name, svc),
            "process" => start_process(name, svc),
            unknown => {
                eprintln!("  error: unknown service type `{}`", unknown);
            }
        }
    }

    println!();
}

fn start_docker(name: &str, project: &str, svc: &config::ServiceConfig) {
    let image = match &svc.image {
        Some(img) => img.clone(),
        None => {
            eprintln!("  error: `{}` has no image defined", name);
            return;
        }
    };

    let container_name = format!("devctl-{}-{}", project, name);

    print!("  →  {:<12} starting...  ", name);

    let mut cmd = Command::new("docker");
    cmd.args(["run", "--detach", "--name", &container_name]);

    if let Some(port) = svc.port {
        cmd.args(["-p", &format!("{}:{}", port, port)]);
    }

    cmd.arg(&image);

    match cmd.output() {
        Ok(output) if output.status.success() => {
            println!("{}", "[ok]".green());
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // 이미 같은 이름의 컨테이너가 있는 경우
            if stderr.contains("already in use") {
                println!("{}", "[ok]".green());
            } else {
                println!("{}", "[failed]".red());
                eprintln!("     {}", stderr.trim().dimmed());
            }
        }
        Err(e) => {
            println!("{}", "[failed]".red());
            eprintln!("     {}", e);
        }
    }
}

fn start_process(name: &str, svc: &config::ServiceConfig) {
    let cmd_str = match &svc.cmd {
        Some(cmd) => cmd.clone(),
        None => {
            eprintln!("  error: `{}` has no cmd defined", name);
            return;
        }
    };

    print!("  →  {:<12} starting...  ", name);

    match Command::new("sh").args(["-c", &cmd_str]).spawn() {
        Ok(_) => println!("{}", "[ok]".green()),
        Err(e) => {
            println!("{}", "[failed]".red());
            eprintln!("     {}", e);
        }
    }
}
