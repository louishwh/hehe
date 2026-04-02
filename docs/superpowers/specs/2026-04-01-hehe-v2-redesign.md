# hehe v2 重构设计

**日期：** 2026-04-01
**目标：** 从零重新设计 hehe，构建一个可插拔的个人 AI 助手平台

---

## 1. 系统定位

一个个人 AI 助手平台，支持：
- 多前端接入（Web、WebSocket、MQTT、各种 IM）
- 可插拔技能（外部 API、脚本、子 Agent）
- 分层记忆（短期/长期/系统化）
- 单机多 Agent，未来可多机互联

---

## 2. Crate 结构与依赖图

```
hehe-core        ← trait + 跨 crate 共享类型，零外部运行时依赖
hehe-llm         ← 依赖 core
hehe-memory      ← 依赖 core
hehe-tools       ← 依赖 core
hehe-agent       ← 依赖 core + llm + memory + tools
hehe-company     ← 依赖 core + agent
hehe-gateway     ← 依赖 core + agent + company
```

严格单向无环。

---

## 3. hehe-core：trait + 共享类型

**入场标准：被 2 个以上 crate 的公共接口引用。**

### 3.1 共享类型

```
core/
├── types/
│   ├── id.rs          # Id（UUID v7 包装）+ AgentId/SessionId/MessageId 等别名
│   ├── timestamp.rs   # Timestamp（chrono DateTime 包装）
│   └── metadata.rs    # Metadata（HashMap<String, Value>）
├── message/
│   ├── role.rs        # Role { System, User, Assistant, Tool }
│   ├── content.rs     # ContentBlock { Text, Image, Audio, Video, File, ToolUse, ToolResult }
│   └── message.rs     # Message { id, role, content, created_at, metadata }
├── stream.rs          # StreamChunk 枚举 + StopReason + StreamAggregator
├── tool.rs            # ToolDefinition + ToolParameter（JSON Schema）
└── error.rs           # Error enum + Result 别名
```

### 3.2 核心 Trait

```rust
// --- LLM 能力边界 ---
#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;
    async fn complete_stream(&self, request: CompletionRequest) -> Result<BoxStream<StreamChunk>>;
    fn default_model(&self) -> &str;
}

// --- 记忆能力边界 ---
#[async_trait]
pub trait MemoryStore: Send + Sync {
    async fn store(&self, entry: MemoryEntry) -> Result<Id>;
    async fn recall(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>>;
    async fn get(&self, id: &Id) -> Result<Option<MemoryEntry>>;
    async fn delete(&self, id: &Id) -> Result<bool>;
    async fn search(&self, filter: MemoryFilter) -> Result<Vec<MemoryEntry>>;
}

// --- 工具能力边界 ---
#[async_trait]
pub trait Tool: Send + Sync {
    fn definition(&self) -> ToolDefinition;
    async fn execute(&self, ctx: &Context, input: Value) -> Result<ToolOutput>;
}

// --- Agent 能力边界 ---
#[async_trait]
pub trait AgentRuntime: Send + Sync {
    fn id(&self) -> &AgentId;
    fn name(&self) -> &str;
    async fn process(&self, session: &Session, input: &str) -> Result<AgentResponse>;
    fn process_stream(&self, session: &Session, input: &str) -> BoxStream<AgentEvent>;
}
```

### 3.3 跨 crate 共享的辅助类型

```rust
// LLM 相关（llm + agent 都用）
pub struct CompletionRequest { model, messages, system, tools, max_tokens, temperature, stream, ... }
pub struct CompletionResponse { id, model, message, stop_reason, usage }
pub struct TokenUsage { input_tokens, output_tokens }

// 工具相关（tools + agent 都用）
pub struct ToolOutput { content: String, is_error: bool, artifacts: Vec<Artifact> }
pub struct Context { request_id, agent_id, session_id, deadline, cancellation_token }

// 记忆相关（memory + agent 都用）
pub struct MemoryEntry { id, kind, content, embedding, metadata, created_at }
pub enum MemoryKind { ShortTerm, LongTerm, System, Episodic }
pub struct MemoryFilter { kind, time_range, query, limit }

// Agent 相关（agent + company + gateway 都用）
pub struct Session { id, messages, stats }
pub struct AgentResponse { session_id, text, tool_calls, iterations }
pub enum AgentEvent { MessageStart, TextDelta, TextComplete, ToolUseStart, ToolUseEnd, ... }
```

### 3.4 不在 core 里的

| 不放 core | 原因 | 去向 |
|-----------|------|------|
| Config/配置加载 | 只有启动入口用 | hehe-gateway 或独立 config 模块 |
| EventEmitter 总线 | 运行时基础设施 | hehe-agent |
| ResourceResolver/ResourceStore | 存储逻辑 | hehe-memory |
| Capabilities 枚举 | 只有 llm 内部用 | hehe-llm |
| Version | CLI 关注 | hehe-gateway |

---

## 4. hehe-llm：LLM Provider 实现

```
llm/
├── provider/
│   ├── openai.rs       # OpenAI（GPT-4o 等）
│   ├── anthropic.rs    # Anthropic（Claude 系列）
│   └── openai_compat.rs # OpenAI 兼容协议（DeepSeek、Qwen、Moonshot、Ollama、LiteLLM）
├── pool.rs             # 连接池：多 provider 管理、fallback、负载均衡
├── retry.rs            # 重试逻辑：指数退避、rate limit 处理
└── types.rs            # Provider 内部类型（OpenAI/Anthropic 的 API 结构体）
```

**设计要点：**
- 实现 `core::LlmProvider` trait
- `openai_compat.rs` 复用 OpenAI 协议实现 DeepSeek/Qwen/Ollama 等，只需改 `base_url` + 模型列表
- `pool.rs` 提供 `LlmPool`，对上层暴露单一 `LlmProvider`，内部做 provider 选择/fallback
- feature gate 控制编译：`openai`、`anthropic`、`deepseek`、`ollama` 等

---

## 5. hehe-memory：记忆系统

```
memory/
├── short_term.rs       # 短期记忆：当前会话的消息窗口（内存，带滑动窗口）
├── long_term.rs        # 长期记忆：对话历史 + 学到的知识（SQLite + 向量索引）
├── system.rs           # 系统记忆：用户偏好、Agent 配置、技能目录（SQLite）
├── router.rs           # 记忆路由：recall 时同时查询多层记忆，按相关性合并
├── embedding.rs        # 嵌入计算：调用 LLM embedding API 或本地模型
└── backend/
    ├── sqlite.rs       # SQLite 后端
    └── memory_vec.rs   # 内存向量存储（开发/测试用）
```

**设计要点：**
- 实现 `core::MemoryStore` trait
- `router.rs` 是关键——`recall("用户之前说过什么？")` 会同时查短期（精确匹配最近 N 条）+ 长期（向量相似度搜索）+ 系统（关键字匹配偏好），然后按相关性排序合并返回
- 短期记忆纯内存，进程级生命周期
- 长期记忆持久化到 SQLite + 向量索引
- 系统记忆用 SQLite key-value 表

---

## 6. hehe-tools：工具执行

```
tools/
├── registry.rs         # ToolRegistry：注册、查找、列出工具
├── executor.rs         # ToolExecutor：并发执行、超时控制
├── sandbox.rs          # 沙箱：危险操作隔离
├── builtin/
│   ├── fs.rs           # read_file, write_file, list_directory, search_files
│   ├── shell.rs        # execute_shell
│   ├── http.rs         # http_request
│   └── system.rs       # get_system_info
└── external/
    ├── script.rs       # 脚本工具：调用外部 Python/Node/Shell 脚本
    └── api.rs          # API 工具：调用外部 REST 服务
```

**设计要点：**
- 实现 `core::Tool` trait
- `executor.rs` 使用 `futures::future::join_all` 并发执行多个工具
- `script.rs`：用户写一个脚本文件 + 描述文件，自动注册为工具
- `api.rs`：用户配置一个 REST endpoint + 参数 schema，自动注册为工具
- 不涉及 Agent 调度——子 Agent 作为工具的能力在 hehe-agent 层实现

---

## 7. hehe-agent：单 Agent 运行时

```
agent/
├── agent.rs            # Agent 结构体 + Builder
├── executor.rs         # ReAct 循环（同步 + 流式）
├── session.rs          # Session 管理（内存 + 可选持久化）
├── planner.rs          # 决策逻辑：Reply / UseTool / Delegate
└── event.rs            # AgentEvent 发送逻辑
```

**核心循环（executor.rs）：**

```rust
loop {
    // 1. 回忆相关记忆
    let context = memory.recall(&input, limit).await;

    // 2. 构建 LLM 请求（消息 + 记忆上下文 + 可用工具）
    let request = build_request(&session, &context, &tools);

    // 3. LLM 决策
    let response = llm.complete_stream(request).await;

    // 4. 处理响应
    match classify(&response) {
        Plan::Reply(text) => {
            memory.store(entry).await;
            return text;
        }
        Plan::UseTools(calls) => {
            let results = executor.execute_concurrent(calls).await;
            session.add_tool_results(results);
            // 继续循环
        }
        Plan::Delegate(agent_id, msg) => {
            // 通过 company 层的 router 委派
            let result = router.send(agent_id, msg).await;
            session.add_delegation_result(result);
            // 继续循环
        }
    }
}
```

**设计要点：**
- 实现 `core::AgentRuntime` trait
- 流式实现是真正的 token 级 streaming：循环中调 `complete_stream()`，实时转发 `TextDelta`
- Agent 可以把另一个 Agent 包装成 Tool 使用（`AgentAsTool` 适配器）
- Session 支持可选的 SQLite 持久化（通过 `MemoryStore` 的系统记忆层）

---

## 8. hehe-company：多 Agent 协作

```
company/
├── company.rs          # Company 结构体：管理一组 Agent
├── router.rs           # 消息路由：用户消息 → 匹配 Agent
├── delegation.rs       # Agent 间委派协议
└── topology.rs         # 协作拓扑：管道、星型、层级
```

**设计要点：**
- `Company` 持有多个 `Arc<dyn AgentRuntime>`
- `Router` 根据策略分发消息：关键词匹配、LLM 分类、显式指定
- 支持三种协作模式：
  - **管道**：A → B → C 顺序处理
  - **星型**：主 Agent 调度多个子 Agent
  - **层级**：主 Agent 委派给专家 Agent，专家可继续委派
- 第一版先实现星型模式（主 Agent + 子 Agent），最通用

---

## 9. hehe-gateway：协议接入层

```
gateway/
├── main.rs             # 启动入口
├── config.rs           # 全局配置加载
├── transport/
│   ├── trait.rs        # Transport trait
│   ├── websocket.rs    # WebSocket 接入
│   ├── http.rs         # HTTP/SSE 接入
│   └── mqtt.rs         # MQTT 接入
├── adapter/
│   └── im.rs           # IM 适配器（微信、钉钉、Telegram）
└── server.rs           # 统一服务启动（多协议并行监听）
```

**设计要点：**
- Transport trait：`receive() -> Stream<IncomingMessage>` + `send(OutgoingMessage)`
- 所有 Transport 统一转为内部 `Message`，调用 `Company` 或 `Agent` 处理
- Gateway 是唯一的 `bin` crate，其余全是 `lib`
- 配置、版本信息、CLI 参数解析都放这里

---

## 10. 与现有 hehe 的对比

| 维度 | 现有 hehe | v2 |
|------|-----------|-----|
| crate 数量 | 7 个 | 7 个（但每个有清晰边界） |
| core 内容 | trait + 类型 + 配置 + 事件总线 + 资源管理 | 仅 trait + 跨 crate 共享类型 |
| Streaming | 伪流式 | 真正 token 级流式 |
| 工具执行 | 串行 | 并发（join_all） |
| LLM Provider | 仅 OpenAI | OpenAI + Anthropic + OpenAI 兼容协议 |
| 记忆 | 无（Session 纯内存） | 短期 + 长期 + 系统化，向量搜索 |
| 多 Agent | 无 | Company 层管理，支持委派 |
| 接入协议 | 仅 HTTP | WebSocket + HTTP + MQTT + IM |
| 技能扩展 | 仅内置 Tool | 内置 + 脚本 + API + 子 Agent |

---

## 11. 可复用的现有代码

| 现有文件 | 可复用程度 | 说明 |
|----------|-----------|------|
| `message/content.rs` | 高 | ContentBlock、Source、ToolUse、ToolResult 设计良好 |
| `message/message.rs` | 高 | Message 结构体 + Builder |
| `types/id.rs` | 高 | UUID v7 包装，直接复用 |
| `types/timestamp.rs` | 高 | 直接复用 |
| `types/metadata.rs` | 高 | 直接复用 |
| `stream/mod.rs` | 高 | StreamChunk + StreamAggregator |
| `tool/schema.rs` | 高 | ToolDefinition + ToolParameter |
| `llm/providers/openai.rs` | 中 | 需要精简 serde 结构体，但核心逻辑可复用 |
| `tools/builtin/*` | 中 | 工具实现可复用，注册机制重写 |
| `store/sqlite.rs` | 中 | SQLite 包装可复用到 memory 层 |
| `config/*` | 低 | 配置结构需要完全重新设计 |
| `capability/*` | 低 | 迁入 hehe-llm 内部，不再跨 crate |
| `event/*` | 低 | 事件总线迁入 hehe-agent |
| `resource/*` | 低 | 迁入 hehe-memory |

---

## 12. 实现顺序

1. **hehe-core**：定义所有 trait 和共享类型
2. **hehe-llm**：OpenAI provider（复用现有代码）+ Anthropic
3. **hehe-memory**：短期记忆（内存）+ 长期记忆（SQLite）
4. **hehe-tools**：复用现有内置工具 + 脚本/API 扩展
5. **hehe-agent**：单 Agent 运行时 + 真正流式
6. **hehe-company**：星型协作模式
7. **hehe-gateway**：WebSocket + HTTP 接入

每一步都可独立编译、独立测试。
