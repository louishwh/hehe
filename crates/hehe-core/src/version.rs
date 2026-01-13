pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const MESSAGE_FORMAT_VERSION: u32 = 1;
pub const CONFIG_FORMAT_VERSION: u32 = 1;
pub const STORAGE_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug)]
pub struct VersionInfo {
    pub version: &'static str,
    pub message_format: u32,
    pub config_format: u32,
    pub storage_schema: u32,
    pub git_hash: Option<&'static str>,
    pub build_time: Option<&'static str>,
}

impl VersionInfo {
    pub fn current() -> Self {
        Self {
            version: VERSION,
            message_format: MESSAGE_FORMAT_VERSION,
            config_format: CONFIG_FORMAT_VERSION,
            storage_schema: STORAGE_SCHEMA_VERSION,
            git_hash: option_env!("GIT_HASH"),
            build_time: option_env!("BUILD_TIME"),
        }
    }
}
