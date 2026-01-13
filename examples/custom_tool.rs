//! Custom tool example
//!
//! Run with: cargo run --example custom_tool

use async_trait::async_trait;
use hehe_agent::Agent;
use hehe_core::{Context, ToolDefinition, ToolParameter};
use hehe_llm::OpenAiProvider;
use hehe_tools::{Result, Tool, ToolOutput, ToolRegistry};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// A custom calculator tool
struct CalculatorTool {
    def: ToolDefinition,
}

impl CalculatorTool {
    fn new() -> Self {
        let def = ToolDefinition::new("calculator", "Perform basic arithmetic operations")
            .with_required_param(
                "operation",
                ToolParameter::string()
                    .with_description("Operation: add, subtract, multiply, divide"),
            )
            .with_required_param("a", ToolParameter::number().with_description("First number"))
            .with_required_param("b", ToolParameter::number().with_description("Second number"));
        Self { def }
    }
}

#[derive(Deserialize)]
struct CalculatorInput {
    operation: String,
    a: f64,
    b: f64,
}

#[async_trait]
impl Tool for CalculatorTool {
    fn definition(&self) -> &ToolDefinition {
        &self.def
    }

    async fn execute(&self, _ctx: &Context, input: Value) -> Result<ToolOutput> {
        let input: CalculatorInput = serde_json::from_value(input)?;

        let result = match input.operation.as_str() {
            "add" => input.a + input.b,
            "subtract" => input.a - input.b,
            "multiply" => input.a * input.b,
            "divide" => {
                if input.b == 0.0 {
                    return Ok(ToolOutput::error("Division by zero"));
                }
                input.a / input.b
            }
            _ => return Ok(ToolOutput::error(format!("Unknown operation: {}", input.operation))),
        };

        Ok(ToolOutput::text(format!("{}", result)))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY environment variable required");

    let llm = Arc::new(OpenAiProvider::new(api_key));

    // Create registry with custom tool
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(CalculatorTool::new()))?;

    let agent = Agent::builder()
        .system_prompt("You are a math assistant. Use the calculator tool for arithmetic.")
        .model("gpt-4o")
        .llm(llm)
        .tool_registry(Arc::new(registry))
        .build()?;

    let session = agent.create_session();

    let response = agent
        .process(&session, "What is 42 multiplied by 17?")
        .await?;

    println!("Response: {}", response.text());

    for tc in &response.tool_calls {
        println!("Tool: {} -> {}", tc.name, tc.output);
    }

    Ok(())
}
