# CLAUDE.md

本文件为 Claude Code (claude.ai/code) 在本仓库中工作时提供指导。

## 项目概述

这是一个基于 Rust 的 SARIF（静态分析结果交换格式）解析器和 SAST（静态应用安全测试）结果验证工具。该工具处理来自 CodeQL、Semgrep 等静态分析工具的安全扫描输出（支持 SARIF 格式），并集成 LLM API 实现安全发现的 AI 分析。

## 构建和运行命令

```bash
# 构建项目
cargo build

# 运行项目
cargo run

# 发布版本构建（优化）
cargo build --release

# 仅检查编译错误，不构建
cargo check

# 运行测试（待添加）
cargo test
```

## 架构

### 模块结构
```
src/
├── main.rs          # 入口文件，异步主函数，使用 tokio 运行时
├── lib.rs           # 库根模块，导出 ai 和 sarif_reader 模块
├── ai.rs            # LLM API 集成（async-openai）
└── sarif_reader.rs  # SARIF 解析和处理
```

### 核心数据流
应用遵循以下处理流程：
1. **SARIF 解析**：使用 `sarif_rust` crate 将 SARIF 文件反序列化为 Rust 结构体
2. **结果提取**：遍历每次运行（runs）及其结果（results，发现项）
3. **AI 分析**：使用 LLM API（MiniMax）分析和验证安全发现
4. **信息提取**：对每个安全发现提取：
   - 问题类型和消息
   - 规则标识符和位置
   - 代码流（污点传播路径），展示从源到汇的数据流

### 核心依赖
- `sarif_rust` (0.3.0)：提供 SARIF schema 类型和反序列化功能
- `serde_json` (1.0)：用于 SARIF 处理的 JSON 序列化/反序列化
- `async-openai` (0.33)：异步 OpenAI 兼容 API 客户端，用于 LLM 集成
- `tokio` (1)：异步运行时，支持 async/await

`sarif_rust` 主要类型：
- `SarifLog`：根结构，包含一次或多次分析运行
- `Run`：单次工具执行及其结果
- `Result`：单个安全发现，包含位置、消息和代码流
- `CodeFlow`：污点分析路径，展示数据流

### AI 模块（`src/ai.rs`）
- `get_a_client()`：创建配置好 MiniMax API 的异步 OpenAI 兼容客户端
- `chat_with_model()`：向 LLM 发送消息并返回响应

### SARIF 阅读器模块（`src/sarif_reader.rs`）
- 处理 SARIF 文件解析和结果处理
- 工具函数如 `remove_nulls()` 用于清理 JSON 输出

### 当前实现说明
- SARIF 文件路径目前在 `src/main.rs` 中硬编码（`SARIF_LOG` 常量）
- 工具默认处理并打印每个运行的前 3 个结果
- 输出重点关注：消息、规则 ID、位置和污点流步骤
- 异步操作使用 tokio 运行时和 `#[tokio::main]`
- LLM 集成使用 OpenAI 兼容接口连接 MiniMax API
