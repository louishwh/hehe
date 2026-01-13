use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    TextInput,
    ImageInput,
    AudioInput,
    VideoInput,
    FileInput,
    TextOutput,
    ImageOutput,
    AudioOutput,
    ToolUse,
    Streaming,
    JsonMode,
    SystemPrompt,
    MultiTurn,
    CodeExecution,
    WebBrowsing,
    FunctionCalling,
    Vision,
    Custom(String),
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Capabilities {
    #[serde(default)]
    inner: HashSet<Capability>,
}

impl Capabilities {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, cap: Capability) -> Self {
        self.inner.insert(cap);
        self
    }

    pub fn add(&mut self, cap: Capability) -> bool {
        self.inner.insert(cap)
    }

    pub fn remove(&mut self, cap: &Capability) -> bool {
        self.inner.remove(cap)
    }

    pub fn has(&self, cap: &Capability) -> bool {
        self.inner.contains(cap)
    }

    pub fn has_all(&self, caps: &[Capability]) -> bool {
        caps.iter().all(|c| self.has(c))
    }

    pub fn has_any(&self, caps: &[Capability]) -> bool {
        caps.iter().any(|c| self.has(c))
    }

    pub fn iter(&self) -> impl Iterator<Item = &Capability> {
        self.inner.iter()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn text_basic() -> Self {
        Self::new()
            .with(Capability::TextInput)
            .with(Capability::TextOutput)
            .with(Capability::Streaming)
            .with(Capability::SystemPrompt)
            .with(Capability::MultiTurn)
    }

    pub fn vision() -> Self {
        Self::text_basic()
            .with(Capability::ImageInput)
            .with(Capability::Vision)
    }

    pub fn multimodal() -> Self {
        Self::vision()
            .with(Capability::AudioInput)
            .with(Capability::FileInput)
    }

    pub fn tool_capable() -> Self {
        Self::text_basic()
            .with(Capability::ToolUse)
            .with(Capability::FunctionCalling)
    }

    pub fn full_agent() -> Self {
        Self::multimodal()
            .with(Capability::ToolUse)
            .with(Capability::FunctionCalling)
            .with(Capability::JsonMode)
    }

    pub fn merge(&mut self, other: &Capabilities) {
        self.inner.extend(other.inner.iter().cloned());
    }

    pub fn intersection(&self, other: &Capabilities) -> Capabilities {
        Capabilities {
            inner: self.inner.intersection(&other.inner).cloned().collect(),
        }
    }
}

pub trait CapabilityProvider {
    fn capabilities(&self) -> &Capabilities;

    fn supports(&self, cap: &Capability) -> bool {
        self.capabilities().has(cap)
    }

    fn supports_all(&self, caps: &[Capability]) -> bool {
        self.capabilities().has_all(caps)
    }

    fn supports_any(&self, caps: &[Capability]) -> bool {
        self.capabilities().has_any(caps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capabilities() {
        let caps = Capabilities::text_basic();
        assert!(caps.has(&Capability::TextInput));
        assert!(caps.has(&Capability::TextOutput));
        assert!(!caps.has(&Capability::ImageInput));
    }

    #[test]
    fn test_full_agent() {
        let caps = Capabilities::full_agent();
        assert!(caps.has(&Capability::ToolUse));
        assert!(caps.has(&Capability::ImageInput));
        assert!(caps.has(&Capability::Streaming));
    }
}
