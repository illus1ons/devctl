use std::process::Command;

use crate::check::{Check, CheckResult};

pub struct ToolCheck {
    tool: String,
}

impl ToolCheck {
    pub fn new(tool: impl Into<String>) -> Self {
        Self { tool: tool.into() }
    }
}

impl Check for ToolCheck {
    fn name(&self) -> &str {
        &self.tool
    }

    fn run(&self) -> CheckResult {
        let found = Command::new("which")
            .arg(&self.tool)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if found {
            CheckResult::Ok
        } else {
            CheckResult::Error {
                message: format!("`{}` is not installed", self.tool),
                fix: Some(format!("install `{}`", self.tool)),
            }
        }
    }
}
