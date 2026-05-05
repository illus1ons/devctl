use doctor::{
    checks::{env::EnvCheck, port::PortCheck, tool::ToolCheck},
    run, Check,
};

use crate::output::print_reports;

pub fn execute() {
    println!("[devctl] Checking environment...");

    let checks = build_checks();

    if checks.is_empty() {
        println!("  No checks defined. Add services or tools to dev.toml.");
        return;
    }

    let reports = run(checks);
    print_reports(&reports);
}

fn build_checks() -> Vec<Box<dyn Check>> {
    let mut checks: Vec<Box<dyn Check>> = Vec::new();

    match config::load_dev_config() {
        Ok(cfg) => {
            if let Some(doctor) = cfg.doctor {
                for tool in doctor.tools.unwrap_or_default() {
                    checks.push(Box::new(ToolCheck::new(tool)));
                }
            }

            let mut services: Vec<_> = cfg.services.unwrap_or_default().into_iter().collect();
            services.sort_by_key(|(name, _)| name.clone());

            for (name, svc) in services {
                if let Some(port) = svc.port {
                    checks.push(Box::new(PortCheck::in_use(
                        format!("{} (port {})", name, port),
                        port,
                    )));
                }
            }
        }
        Err(config::ConfigError::NotFound(_)) => {
            eprintln!("  warning: dev.toml not found. Run `devctl init` to set up.");
        }
        Err(e) => {
            eprintln!("  error: failed to load dev.toml — {}", e);
        }
    }

    if let Ok(schema) = config::load_env_schema() {
        let mut vars: Vec<_> = schema.into_iter().collect();
        vars.sort_by_key(|(key, _)| key.clone());

        for (key, var) in vars {
            if var.required.unwrap_or(false) {
                checks.push(Box::new(EnvCheck::new(key)));
            }
        }
    }

    checks
}
