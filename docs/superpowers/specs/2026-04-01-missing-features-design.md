# hehe 缺失功能补充设计

**日期：** 2026-04-01  
**范围：** 真正的 Token 级 Streaming、工具并发执行、Anthropic Claude Provider、可选 SQLite Session 持久化

---

## 一、真正的 Token 级 Streaming

### 问题

`Executor::execute_stream` 当前实现是对非流式 `execute` 的包装，等待整个响应完成后才发送 `TextComplete`，从不发送 `TextDelta`。这与 README 声称的 SSE Streaming 不符。

### 设计

修改 `Executor::execute_stream`，在 ReAct 循环中调用 `llm.complete_stream()` 而非 `complete()`。

**数据流：**

```
LLM.complete_stream()
  → StreamChunk::TextDelta         → AgentEvent::TextDelta      → mpsc::Sender
  → StreamChunk::ToolUseStart/Delta → 用 StreamAggregator 聚合
  → StreamChunk::MessageEnd(ToolUse)→ 执行工具
                                      → AgentEvent::ToolUseStart
                                      → AgentEvent::ToolUseEnd
                                     → 继续下一轮 complete_stream()
  → StreamChunk::MessageEnd(EndTurn)→ AgentEvent::TextComplete
                                    → AgentEvent::MessageEnd
```

**关键实现细节：**

- `StreamAggregator`（已在 `hehe-core` 实现）用于聚合每轮流式输出，完成后构建完整的 `Message` 追加到 `Session`
- 工具执行期间停止当前流式（自然结束），执行完毕后开启新一轮 `complete_stream`
- 非流式 `execute` 保持不变，两个方法独立实现，不互相依赖
- 新增私有方法 `execute_stream_inner` 处理单轮流式逻辑

**涉及文件：**
- `crates/hehe-agent/src/executor.rs`：重写 `execute_stream`

---

## 二、工具并发执行

### 问题

`Executor::execute_tools` 串行遍历 `tool_uses`，多工具调用时延迟线性累加。

### 设计

使用 `futures::future::join_all` 将所有工具调用转为并发 future 一次性 await。

```rust
async fn execute_tools(&self, tool_uses: &[&ToolUse]) -> Vec<(String, u64, bool)> {
    let futures = tool_uses.iter().map(|tu| self.execute_single_tool(tu));
    futures::future::join_all(futures).await
}
```

- `join_all` 保证结果顺序与输入顺序一致，调用方无感知变化
- 危险工具与非危险工具均并发——工具 timeout/sandbox 已在 `ToolExecutor` 层控制
- 接口签名 `execute_tools` 保持不变

**涉及文件：**
- `crates/hehe-agent/src/executor.rs`：修改 `execute_tools`，提取 `execute_single_tool`

---

## 三、Anthropic Claude Provider

### 问题

`hehe-llm/Cargo.toml` 已声明 `anthropic` feature，但 `providers/` 目录仅有 `openai.rs`，`anthropic` feature 是空声明。

### 设计

新增 `crates/hehe-llm/src/providers/anthropic.rs`，实现 `LlmProvider` trait。

**API 映射：**

| hehe 概念 | Anthropic API |
|-----------|--------------|
| `complete()` | `POST /v1/messages` |
| `complete_stream()` | `POST /v1/messages` with `"stream": true`（SSE） |
| `list_models()` | 静态列表（Anthropic 无 list models API） |

**消息格式转换（与 OpenAI 的关键差异）：**

- system prompt 作为顶层 `system` 字段，不进入 `messages` 数组
- 工具调用响应：`Role::Assistant` + `content: [{type: "tool_use", id, name, input}]`
- 工具结果：`Role::User` + `content: [{type: "tool_result", tool_use_id, content}]`
- Anthropic 要求 `max_tokens` 必填（默认 4096）

**流式 SSE 解析：**

- 事件类型：`content_block_start`、`content_block_delta`（`text_delta` / `input_json_delta`）、`message_stop`
- 工具调用通过 `input_json_delta` 增量聚合

**配置：**
```rust
pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    base_url: String,       // 默认 https://api.anthropic.com/v1
    default_model: String,  // 默认 claude-3-5-sonnet-20241022
    capabilities: Capabilities,
}

impl AnthropicProvider {
    pub fn new(api_key: impl Into<String>) -> Self
    pub fn with_base_url(api_key, base_url) -> Self
    pub fn with_model(self, model) -> Self
}
```

**请求头：**
```
x-api-key: {api_key}
anthropic-version: 2023-06-01
content-type: application/json
```

**默认支持的模型（静态）：**
- `claude-3-5-sonnet-20241022`（默认）
- `claude-3-5-haiku-20241022`
- `claude-3-opus-20240229`

**涉及文件：**
- `crates/hehe-llm/src/providers/anthropic.rs`：新增
- `crates/hehe-llm/src/providers/mod.rs`：导出 `AnthropicProvider`
- `crates/hehe-llm/src/lib.rs`：`#[cfg(feature = "anthropic")] pub use providers::AnthropicProvider`

---

## 四、可选 SQLite Session 持久化

### 问题

`Session` 完全基于内存，进程重启后丢失。`hehe-store` 已实现 `SqliteStore`，但未被 `hehe-agent` 使用。

### 设计

**新增 `SessionStore` trait（在 `hehe-agent/src/store.rs`）：**

```rust
#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn save(&self, session: &Session) -> Result<()>;
    async fn load(&self, id: &Id) -> Result<Option<Session>>;
    async fn delete(&self, id: &Id) -> Result<()>;
    async fn list(&self) -> Result<Vec<Id>>;
}
```

**`SqliteSessionStore` 实现（在 `hehe-agent/src/store.rs`）：**

SQLite Schema（通过 migration 初始化）：
```sql
CREATE TABLE sessions (
    id          TEXT PRIMARY KEY,
    created_at  TEXT NOT NULL,
    messages    TEXT NOT NULL,   -- JSON array of Message
    stats       TEXT NOT NULL    -- JSON SessionStats
);
```

- `save`：UPSERT（`INSERT OR REPLACE`）
- `load`：查询后反序列化 messages JSON
- 序列化用已有的 `serde` derive（`Message`、`SessionStats` 均已 derive `Serialize/Deserialize`）

**`Agent` 集成：**

```rust
pub struct Agent {
    config: AgentConfig,
    llm: Arc<dyn LlmProvider>,
    tools: Option<Arc<ToolExecutor>>,
    session_store: Option<Arc<dyn SessionStore>>,  // 新增，可选
}
```

新增方法：
```rust
impl Agent {
    pub async fn load_session(&self, id: &Id) -> Result<Option<Session>>
    pub async fn save_session(&self, session: &Session) -> Result<()>
}
```

`process` 和 `chat` 在完成后，若 `session_store` 存在则自动 `save_session`。

`AgentBuilder` 新增：
```rust
pub fn session_store(mut self, store: Arc<dyn SessionStore>) -> Self
```

**涉及文件：**
- `crates/hehe-agent/src/store.rs`：新增，含 `SessionStore` trait 和 `SqliteSessionStore`
- `crates/hehe-agent/src/lib.rs`：导出 `SessionStore`、`SqliteSessionStore`
- `crates/hehe-agent/src/agent.rs`：`Agent` 增加 `session_store` 字段，`AgentBuilder` 增加 `session_store` 方法
- `crates/hehe-agent/src/executor.rs`：`execute` 返回后 save（或在 agent.rs 层处理）
- `crates/hehe-agent/Cargo.toml`：增加 `hehe-store` 依赖（feature-gated `sqlite`）

---

## 受影响文件汇总

| 文件 | 变更类型 |
|------|---------|
| `crates/hehe-agent/src/executor.rs` | 修改（streaming + 工具并发） |
| `crates/hehe-agent/src/agent.rs` | 修改（session store 集成） |
| `crates/hehe-agent/src/store.rs` | 新增 |
| `crates/hehe-agent/src/lib.rs` | 修改（导出 store） |
| `crates/hehe-agent/Cargo.toml` | 修改（添加 hehe-store 依赖） |
| `crates/hehe-llm/src/providers/anthropic.rs` | 新增 |
| `crates/hehe-llm/src/providers/mod.rs` | 修改（导出 Anthropic） |
| `crates/hehe-llm/src/lib.rs` | 修改（feature-gate 导出） |

---

## 不变的部分

- `hehe-core`：无需修改，`StreamAggregator`、`StreamChunk`、`AgentEvent` 均已满足需求
- `hehe-tools`：无需修改
- `hehe-server`：无需修改（Server 层 SSE 已结构正确，底层 streaming 修复后自动受益）
- `hehe-cli`：无需修改
- 所有公开 API 向后兼容（新增字段均为可选）

---

## 测试策略

每个模块均沿用现有 MockLlm 模式：

1. **Streaming**：MockLlm 的 `complete_stream` 发出真实 `StreamChunk` 序列，断言 `AgentEvent::TextDelta` 在 `TextComplete` 之前收到
2. **工具并发**：记录工具执行时间戳，断言多个工具的执行时间窗口有重叠
3. **Anthropic Provider**：单元测试消息格式转换（不需要真实 API），集成测试可选
4. **Session 持久化**：使用 `SqliteSessionStore::memory()` 测试 save/load 往返
