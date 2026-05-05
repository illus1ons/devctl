use crate::check::{Check, CheckResult};

pub struct CheckReport {
    pub name: String,
    pub result: CheckResult,
}

pub fn run(checks: Vec<Box<dyn Check>>) -> Vec<CheckReport> {
    checks
        .into_iter()
        .map(|c| CheckReport {
            name: c.name().to_string(),
            result: c.run(),
        })
        .collect()
}
