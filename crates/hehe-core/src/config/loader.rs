use super::types::Config;
use crate::error::{Error, Result};
use std::path::Path;

impl Config {
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())?;
        Self::from_toml(&content)
    }

    pub fn from_toml(content: &str) -> Result<Self> {
        toml::from_str(content).map_err(|e| Error::Config(format!("Failed to parse config: {}", e)))
    }

    pub fn from_json(content: &str) -> Result<Self> {
        serde_json::from_str(content).map_err(Error::Json)
    }

    pub fn load_default() -> Result<Self> {
        let paths = [
            "./hehe.toml",
            "~/.hehe/config.toml",
            "~/.config/hehe/config.toml",
            "/etc/hehe/config.toml",
        ];

        for path in &paths {
            let expanded = shellexpand::tilde(path);
            let path = Path::new(expanded.as_ref());
            if path.exists() {
                return Self::load_from_file(path);
            }
        }

        Ok(Config::default())
    }

    pub fn merge_env(mut self) -> Self {
        if let Ok(level) = std::env::var("HEHE_LOG_LEVEL") {
            self.general.log_level = match level.to_lowercase().as_str() {
                "trace" => super::types::LogLevel::Trace,
                "debug" => super::types::LogLevel::Debug,
                "info" => super::types::LogLevel::Info,
                "warn" => super::types::LogLevel::Warn,
                "error" => super::types::LogLevel::Error,
                _ => self.general.log_level,
            };
        }

        if let Ok(dir) = std::env::var("HEHE_DATA_DIR") {
            self.general.data_dir = dir.into();
        }

        if let Ok(provider) = std::env::var("HEHE_DEFAULT_PROVIDER") {
            self.llm.default_provider = Some(provider);
        }

        self
    }

    pub fn to_toml(&self) -> Result<String> {
        toml::to_string_pretty(self)
            .map_err(|e| Error::Config(format!("Failed to serialize config: {}", e)))
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(Error::Json)
    }

    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let content = self.to_toml()?;
        std::fs::write(path.as_ref(), content)?;
        Ok(())
    }

    pub fn data_dir(&self) -> std::path::PathBuf {
        let expanded = shellexpand::tilde(self.general.data_dir.as_str());
        std::path::PathBuf::from(expanded.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.general.log_level, super::super::types::LogLevel::Info);
    }

    #[test]
    fn test_config_from_toml() {
        let toml = r#"
            [general]
            log_level = "debug"
            
            [llm]
            default_provider = "openai"
            
            [llm.providers.openai]
            provider_type = "openai"
            model = "gpt-4"
        "#;

        let config = Config::from_toml(toml).unwrap();
        assert_eq!(config.general.log_level, super::super::types::LogLevel::Debug);
        assert_eq!(config.llm.default_provider, Some("openai".to_string()));
    }
}
