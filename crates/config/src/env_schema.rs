use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct EnvVarSchema {
    pub required: Option<bool>,
    pub default: Option<String>,
    pub description: Option<String>,
    pub example: Option<String>,
}

pub type EnvSchema = HashMap<String, EnvVarSchema>;
