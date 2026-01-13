//! Example using tools
//!
//! Run with: cargo run --example with_tools

use hehe_agent::Agent;
use hehe_llm::OpenAiProvider;
use hehe_tools::create_default_registry;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY environment variable required");

    let llm = Arc::new(OpenAiProvider::new(api_key));
    let tools = Arc::new(create_default_registry());

    // Create agent with tools
    let agent = Agent::builder()
        .system_prompt("You are a helpful assistant with file system access.")
        .model("gpt-4o")
        .llm(llm)
        .tool_registry(tools)
        .max_iterations(5)
        .build()?;

    let session = agent.create_session();

    // Agent can use tools to answer questions
    let response = agent
        .process(&session, "List the files in the current directory")
        .await?;

    println!("Response: {}", response.text());
    println!("Tool calls: {}", response.tool_call_count());
    println!("Iterations: {}", response.iterations);

    Ok(())
}
