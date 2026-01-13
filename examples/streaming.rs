//! Streaming chat example
//!
//! Run with: cargo run --example streaming

use futures::StreamExt;
use hehe_agent::{Agent, AgentEvent};
use hehe_llm::OpenAiProvider;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY environment variable required");

    let llm = Arc::new(OpenAiProvider::new(api_key));

    let agent = Agent::builder()
        .system_prompt("You are a helpful assistant.")
        .model("gpt-4o")
        .llm(llm)
        .build()?;

    let session = agent.create_session();

    println!("Streaming response:\n");

    let mut stream = agent.chat_stream(&session, "Write a haiku about Rust programming");

    while let Some(event) = stream.next().await {
        match event {
            AgentEvent::TextDelta { delta } => {
                print!("{}", delta);
            }
            AgentEvent::TextComplete { text } => {
                println!("\n\n[Complete: {} chars]", text.len());
            }
            AgentEvent::ToolUseStart { name, .. } => {
                println!("\n[Using tool: {}]", name);
            }
            AgentEvent::ToolUseEnd { output, is_error, .. } => {
                if is_error {
                    println!("[Tool error: {}]", output);
                } else {
                    println!("[Tool result: {}]", output);
                }
            }
            AgentEvent::Error { message } => {
                eprintln!("\n[Error: {}]", message);
            }
            AgentEvent::MessageEnd { .. } => {
                println!("\n[Done]");
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
