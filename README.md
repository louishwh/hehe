# hehe

A modular AI Agent framework in Rust.

## Features

- **Multi-modal Messages**: Text, image, audio, video, file support
- **Tool System**: 7 built-in tools + extensible
- **Storage Abstraction**: SQLite, memory cache, vector store, FTS5
- **Streaming**: SSE + AgentEvent
- **Session Management**: Persistent conversation state
- **Safety**: Dangerous operation marking + sandbox

## Installation

```bash
cargo install --path crates/hehe-cli
```

Or add to your project:

```toml
[dependencies]
hehe-agent = { path = "crates/hehe-agent" }
hehe-llm = { path = "crates/hehe-llm" }
hehe-tools = { path = "crates/hehe-tools" }
```

## Quick Start

### CLI Usage

```bash
# Set API key
export OPENAI_API_KEY=sk-...

# Interactive chat
hehe chat

# Single message
hehe run "What is the capital of France?"

# Start HTTP server
hehe serve --port 3000

# With options
hehe chat --model gpt-4o --system "You are a coding assistant."
```

### Library Usage

```rust
use hehe_agent::Agent;
use hehe_llm::OpenAiProvider;
use hehe_tools::create_default_registry;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let llm = Arc::new(OpenAiProvider::new(std::env::var("OPENAI_API_KEY")?));
    let tools = Arc::new(create_default_registry());

    let agent = Agent::builder()
        .system_prompt("You are a helpful assistant.")
        .model("gpt-4o")
        .llm(llm)
        .tool_registry(tools)
        .build()?;

    let session = agent.create_session();
    let response = agent.chat(&session, "Hello!").await?;
    println!("{}", response);

    Ok(())
}
```

## Architecture

```
hehe/
├── hehe-core      # Base types, messages, events
├── hehe-store     # SQLite, cache, vector, FTS5
├── hehe-llm       # LLM providers (OpenAI)
├── hehe-tools     # Tool system, built-in tools
├── hehe-agent     # Agent runtime, ReAct loop
├── hehe-server    # HTTP/SSE API
└── hehe-cli       # Command-line interface
```

### Dependency Graph

```
                    hehe-cli
                        │
           ┌────────────┼────────────┐
           ▼            ▼            ▼
      hehe-server  hehe-agent   (direct)
           │            │
           └─────┬──────┘
                 ▼
         ┌───────┼───────┐
         ▼       ▼       ▼
    hehe-llm hehe-tools hehe-store
         └───────┼───────┘
                 ▼
            hehe-core
```

## Built-in Tools

| Tool | Description | Dangerous |
|------|-------------|-----------|
| `read_file` | Read file contents | No |
| `write_file` | Write to file | Yes |
| `list_directory` | List directory (recursive) | No |
| `search_files` | Glob pattern search | No |
| `execute_shell` | Execute shell command | Yes |
| `http_request` | HTTP requests | No |
| `get_system_info` | System information | No |

## HTTP API

### Endpoints

```
GET  /health              Health check
GET  /ready               Ready check
POST /api/v1/chat         Sync chat
POST /api/v1/chat/stream  SSE streaming chat
```

### Example

```bash
# Sync chat
curl -X POST http://localhost:3000/api/v1/chat \
  -H "Content-Type: application/json" \
  -d '{"message": "Hello!"}'

# Streaming chat
curl -X POST http://localhost:3000/api/v1/chat/stream \
  -H "Content-Type: application/json" \
  -d '{"message": "Tell me a story"}'
```

## Examples

See the `examples/` directory for more usage examples:

- `basic_chat.rs` - Basic chat example
- `with_tools.rs` - Using tools
- `http_server.rs` - HTTP server setup
- `custom_tool.rs` - Creating custom tools

## License

MIT OR Apache-2.0
