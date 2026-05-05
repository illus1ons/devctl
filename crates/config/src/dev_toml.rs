use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DevConfig {
    pub project: Option<ProjectConfig>,
    pub doctor: Option<DoctorConfig>,
    pub services: Option<HashMap<String, ServiceConfig>>,
}

#[derive(Debug, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct DoctorConfig {
    pub tools: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ServiceConfig {
    pub port: Option<u16>,
}
