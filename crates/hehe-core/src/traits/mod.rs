use crate::error::Result;
use crate::types::Id;

pub trait Identifiable {
    fn id(&self) -> Id;
}

pub trait Validatable {
    fn validate(&self) -> Result<()>;
}

#[async_trait::async_trait]
pub trait Lifecycle: Send + Sync {
    async fn start(&mut self) -> Result<()>;
    async fn stop(&mut self) -> Result<()>;
    fn is_running(&self) -> bool;
}

pub trait Named {
    fn name(&self) -> &str;
}
