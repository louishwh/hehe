//! Basic chat example
//!
//! Run with: cargo run --example basic_chat

use hehe_agent::Agent;
use hehe_llm::OpenAiProvider;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY environment variable required");

    // Create LLM provider
    let llm = Arc::new(OpenAiProvider::new(api_key));

    // Create agent
    let agent = Agent::builder()
        .system_prompt("You are a helpful assistant.")
        .model("gpt-4o")
        .llm(llm)
        .build()?;

    // Create session
    let session = agent.create_session();

    // Chat
    let response = agent.chat(&session, "What is Rust programming language?").await?;
    println!("Assistant: {}", response);

    // Continue conversation (session maintains context)
    let response = agent.chat(&session, "What are its main features?").await?;
    println!("Assistant: {}", response);

    Ok(())
}
