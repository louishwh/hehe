use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentConfig {
    #[serde(default = "default_name")]
    pub name: String,

    pub system_prompt: String,

    pub model: String,

    #[serde(default = "default_temperature")]
    pub temperature: f32,

    #[serde(default = "default_max_tokens")]
    pub max_tokens: Option<usize>,

    #[serde(default = "default_max_iterations")]
    pub max_iterations: usize,

    #[serde(default = "default_max_context_messages")]
    pub max_context_messages: usize,

    #[serde(default = "default_tool_timeout_secs")]
    pub tool_timeout_secs: u64,

    #[serde(default = "default_tools_enabled")]
    pub tools_enabled: bool,
}

fn default_name() -> String {
    "assistant".to_string()
}

fn default_temperature() -> f32 {
    0.7
}

fn default_max_tokens() -> Option<usize> {
    None
}

fn default_max_iterations() -> usize {
    10
}

fn default_max_context_messages() -> usize {
    50
}

fn default_tool_timeout_secs() -> u64 {
    60
}

fn default_tools_enabled() -> bool {
    true
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            name: default_name(),
            system_prompt: String::new(),
            model: "gpt-4o".to_string(),
            temperature: default_temperature(),
            max_tokens: default_max_tokens(),
            max_iterations: default_max_iterations(),
            max_context_messages: default_max_context_messages(),
            tool_timeout_secs: default_tool_timeout_secs(),
            tools_enabled: default_tools_enabled(),
        }
    }
}

impl AgentConfig {
    pub fn new(model: impl Into<String>, system_prompt: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            system_prompt: system_prompt.into(),
            ..Default::default()
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    pub fn with_max_iterations(mut self, max_iterations: usize) -> Self {
        self.max_iterations = max_iterations;
        self
    }

    pub fn with_max_context_messages(mut self, max_messages: usize) -> Self {
        self.max_context_messages = max_messages;
        self
    }

    pub fn with_tool_timeout(mut self, timeout: Duration) -> Self {
        self.tool_timeout_secs = timeout.as_secs();
        self
    }

    pub fn with_tools_enabled(mut self, enabled: bool) -> Self {
        self.tools_enabled = enabled;
        self
    }

    pub fn tool_timeout(&self) -> Duration {
        Duration::from_secs(self.tool_timeout_secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = AgentConfig::default();
        assert_eq!(config.name, "assistant");
        assert_eq!(config.max_iterations, 10);
        assert!(config.tools_enabled);
    }

    #[test]
    fn test_config_builder() {
        let config = AgentConfig::new("gpt-4o", "You are helpful.")
            .with_name("my-agent")
            .with_temperature(0.5)
            .with_max_iterations(5);

        assert_eq!(config.name, "my-agent");
        assert_eq!(config.model, "gpt-4o");
        assert_eq!(config.temperature, 0.5);
        assert_eq!(config.max_iterations, 5);
    }
}
