use std::net::TcpListener;

use crate::check::{Check, CheckResult};

pub struct PortCheck {
    name: String,
    port: u16,
}

impl PortCheck {
    pub fn new(port: u16) -> Self {
        Self {
            name: format!("port {}", port),
            port,
        }
    }
}

impl Check for PortCheck {
    fn name(&self) -> &str {
        &self.name
    }

    fn run(&self) -> CheckResult {
        let addr = format!("127.0.0.1:{}", self.port);
        match TcpListener::bind(&addr) {
            Ok(_) => CheckResult::Ok,
            Err(_) => CheckResult::Warning {
                message: format!("port {} is already in use", self.port),
            },
        }
    }
}
