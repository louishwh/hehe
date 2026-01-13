//! HTTP server example
//!
//! Run with: cargo run --example http_server
//! Test with: curl -X POST http://localhost:3000/api/v1/chat \
//!            -H "Content-Type: application/json" \
//!            -d '{"message": "Hello!"}'

use hehe_agent::Agent;
use hehe_llm::OpenAiProvider;
use hehe_server::{shutdown_signal, Server, ServerConfig};
use hehe_tools::create_default_registry;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let api_key = std::env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY environment variable required");

    let llm = Arc::new(OpenAiProvider::new(api_key));
    let tools = Arc::new(create_default_registry());

    let agent = Agent::builder()
        .system_prompt("You are a helpful assistant.")
        .model("gpt-4o")
        .llm(llm)
        .tool_registry(tools)
        .build()?;

    let config = ServerConfig::new()
        .with_host("127.0.0.1")
        .with_port(3000);

    println!("Starting server on http://127.0.0.1:3000");
    println!("Press Ctrl+C to stop");

    let server = Server::new(config, agent);
    server.run_with_shutdown(shutdown_signal()).await?;

    Ok(())
}
