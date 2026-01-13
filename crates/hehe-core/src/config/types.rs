use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub llm: LlmConfig,
    #[serde(default)]
    pub storage: StorageConfig,
    #[serde(default)]
    pub tools: ToolsConfig,
    #[serde(default)]
    pub security: SecurityConfig,
    #[serde(default)]
    pub telemetry: TelemetryConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            llm: LlmConfig::default(),
            storage: StorageConfig::default(),
            tools: ToolsConfig::default(),
            security: SecurityConfig::default(),
            telemetry: TelemetryConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_data_dir")]
    pub data_dir: Utf8PathBuf,
    #[serde(default)]
    pub log_level: LogLevel,
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent_agents: usize,
}

fn default_data_dir() -> Utf8PathBuf {
    Utf8PathBuf::from("~/.hehe")
}

fn default_max_concurrent() -> usize {
    10
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            data_dir: default_data_dir(),
            log_level: LogLevel::default(),
            max_concurrent_agents: default_max_concurrent(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LlmConfig {
    #[serde(default)]
    pub default_provider: Option<String>,
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
    #[serde(default)]
    pub routing: RoutingConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub provider_type: String,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub max_retries: Option<u32>,
    #[serde(default)]
    pub timeout_secs: Option<u64>,
    #[serde(default)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RoutingConfig {
    #[serde(default)]
    pub strategy: RoutingStrategy,
    #[serde(default)]
    pub fallback_chain: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingStrategy {
    #[default]
    Single,
    RoundRobin,
    LeastLatency,
    CostOptimized,
    Fallback,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StorageConfig {
    #[serde(default)]
    pub relational: StorageBackendConfig,
    #[serde(default)]
    pub columnar: StorageBackendConfig,
    #[serde(default)]
    pub vector: StorageBackendConfig,
    #[serde(default)]
    pub cache: StorageBackendConfig,
    #[serde(default)]
    pub search: StorageBackendConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StorageBackendConfig {
    #[serde(default = "default_backend")]
    pub backend: String,
    #[serde(default)]
    pub connection: Option<String>,
    #[serde(default)]
    pub options: HashMap<String, serde_json::Value>,
}

fn default_backend() -> String {
    "default".to_string()
}

impl Default for StorageBackendConfig {
    fn default() -> Self {
        Self {
            backend: default_backend(),
            connection: None,
            options: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ToolsConfig {
    #[serde(default = "default_true")]
    pub builtin_enabled: bool,
    #[serde(default)]
    pub plugins_dir: Option<Utf8PathBuf>,
    #[serde(default)]
    pub mcp_servers: Vec<McpServerConfig>,
    #[serde(default)]
    pub disabled_tools: Vec<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SecurityConfig {
    #[serde(default)]
    pub mode: SecurityMode,
    #[serde(default = "default_true")]
    pub dangerous_tools_require_confirmation: bool,
    #[serde(default)]
    pub allowed_paths: Vec<Utf8PathBuf>,
    #[serde(default)]
    pub blocked_domains: Vec<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            mode: SecurityMode::default(),
            dangerous_tools_require_confirmation: true,
            allowed_paths: vec![],
            blocked_domains: vec![],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurityMode {
    Strict,
    #[default]
    Normal,
    Autonomous,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TelemetryConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub metrics_endpoint: Option<String>,
    #[serde(default)]
    pub tracing_endpoint: Option<String>,
    #[serde(default)]
    pub log_to_file: bool,
    #[serde(default)]
    pub log_file_path: Option<Utf8PathBuf>,
}
