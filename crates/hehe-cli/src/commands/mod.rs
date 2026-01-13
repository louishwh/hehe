pub mod chat;
pub mod run;
pub mod serve;

use hehe_agent::Agent;
use hehe_llm::OpenAiProvider;
use hehe_tools::create_default_registry;
use std::sync::Arc;

pub fn create_agent(
    api_key: Option<String>,
    model: &str,
    system_prompt: &str,
) -> anyhow::Result<Agent> {
    let api_key = api_key.ok_or_else(|| anyhow::anyhow!(
        "API key required. Set OPENAI_API_KEY env var or use --api-key"
    ))?;

    let llm = Arc::new(OpenAiProvider::new(api_key));
    let registry = Arc::new(create_default_registry());

    let agent = Agent::builder()
        .system_prompt(system_prompt)
        .model(model)
        .llm(llm)
        .tool_registry(registry)
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to create agent: {}", e))?;

    Ok(agent)
}
