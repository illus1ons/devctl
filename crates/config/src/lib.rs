mod dev_toml;
mod env_schema;

pub use dev_toml::{DevConfig, DoctorConfig, ServiceConfig};
pub use env_schema::{EnvSchema, EnvVarSchema};

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Parse(toml::de::Error),
    NotFound(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "IO error: {}", e),
            ConfigError::Parse(e) => write!(f, "Parse error: {}", e),
            ConfigError::NotFound(name) => write!(f, "`{}` not found", name),
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        ConfigError::Io(e)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(e: toml::de::Error) -> Self {
        ConfigError::Parse(e)
    }
}

pub fn load_dev_config() -> Result<DevConfig, ConfigError> {
    let path = find_file("dev.toml")?;
    let content = std::fs::read_to_string(path)?;
    Ok(toml::from_str(&content)?)
}

pub fn load_env_file() -> std::collections::HashMap<String, String> {
    let Ok(path) = find_file(".env") else {
        return std::collections::HashMap::new();
    };

    let Ok(content) = std::fs::read_to_string(path) else {
        return std::collections::HashMap::new();
    };

    content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let (key, value) = line.split_once('=')?;
            Some((key.trim().to_string(), value.trim().to_string()))
        })
        .collect()
}

pub fn load_env_schema() -> Result<EnvSchema, ConfigError> {
    let path = find_file(".env.schema.toml")?;
    let content = std::fs::read_to_string(path)?;
    Ok(toml::from_str(&content)?)
}

fn find_file(name: &str) -> Result<std::path::PathBuf, ConfigError> {
    let mut dir = std::env::current_dir()?;
    loop {
        let candidate = dir.join(name);
        if candidate.exists() {
            return Ok(candidate);
        }
        if !dir.pop() {
            return Err(ConfigError::NotFound(name.to_string()));
        }
    }
}
