use doctor::{
    checks::{env::EnvCheck, port::PortCheck, tool::ToolCheck},
    run, Check,
};

use crate::output::print_reports;

pub fn execute() {
    println!("[devctl] Checking environment...");

    let checks: Vec<Box<dyn Check>> = vec![
        Box::new(ToolCheck::new("git")),
        Box::new(ToolCheck::new("docker")),
        Box::new(PortCheck::new(5432)),
        Box::new(PortCheck::new(6379)),
        Box::new(EnvCheck::new("DATABASE_URL")),
    ];

    let reports = run(checks);
    print_reports(&reports);
}
