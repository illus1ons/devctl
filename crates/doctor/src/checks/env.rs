use crate::check::{Check, CheckResult};

pub struct EnvCheck {
    key: String,
}

impl EnvCheck {
    pub fn new(key: impl Into<String>) -> Self {
        Self { key: key.into() }
    }
}

impl Check for EnvCheck {
    fn name(&self) -> &str {
        &self.key
    }

    fn run(&self) -> CheckResult {
        match std::env::var(&self.key) {
            Ok(val) if !val.is_empty() => CheckResult::Ok,
            Ok(_) => CheckResult::Warning {
                message: format!("`{}` is set but empty", self.key),
            },
            Err(_) => CheckResult::Error {
                message: format!("`{}` is not set", self.key),
                fix: Some(format!("add {} to .env", self.key)),
            },
        }
    }
}
