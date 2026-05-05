use std::net::TcpListener;

use crate::check::{Check, CheckResult};

pub struct PortCheck {
    name: String,
    port: u16,
    expect_in_use: bool,
}

impl PortCheck {
    /// 포트가 비어있어야 정상 (충돌 감지용)
    pub fn free(port: u16) -> Self {
        Self {
            name: format!("port {}", port),
            port,
            expect_in_use: false,
        }
    }

    /// 포트가 사용 중이어야 정상 (서비스 실행 확인용)
    pub fn in_use(name: impl Into<String>, port: u16) -> Self {
        Self {
            name: name.into(),
            port,
            expect_in_use: true,
        }
    }
}

impl Check for PortCheck {
    fn name(&self) -> &str {
        &self.name
    }

    fn run(&self) -> CheckResult {
        let addr = format!("127.0.0.1:{}", self.port);
        let is_in_use = TcpListener::bind(&addr).is_err();

        match (self.expect_in_use, is_in_use) {
            (true, true) => CheckResult::Ok,
            (true, false) => CheckResult::Error {
                message: format!("port {} not in use (service not running?)", self.port),
                fix: None,
            },
            (false, false) => CheckResult::Ok,
            (false, true) => CheckResult::Warning {
                message: format!("port {} is already in use", self.port),
            },
        }
    }
}
