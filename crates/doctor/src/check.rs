pub enum CheckResult {
    Ok,
    Warning { message: String },
    Error { message: String, fix: Option<String> },
}

pub trait Check: Send + Sync {
    fn name(&self) -> &str;
    fn run(&self) -> CheckResult;
}
