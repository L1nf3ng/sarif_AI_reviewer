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

# 运行测试
cargo test
```

## 架构

### 模块结构
```
src/
├── main.rs           # 入口文件，异步主函数，使用 tokio 运行时
├── lib.rs            # 库根模块，导出 ai_chat、sarif_reader 和 source_reader 模块
├── ai_chat.rs        # LLM API 集成（async-openai）
├── sarif_reader.rs   # SARIF 解析和处理
└── source_reader.rs # tree-sitter 源码解析，提取指定行号所在最小作用域代码
```

### 核心数据流
应用遵循以下处理流程：
1. **SARIF 解析**：使用 `sarif_rust` crate 将 SARIF 文件反序列化为 Rust 结构体
2. **结果提取**：遍历每次运行（runs）及其结果（results，发现项）
3. **源码解析**：使用 tree-sitter 解析源码，根据行号提取所在最小作用域（函数）代码
4. **AI 分析**：使用 LLM API（MiniMax）分析和验证安全发现
5. **信息提取**：对每个安全发现提取：
   - 问题类型和消息
   - 规则标识符和位置
   - 代码流（污点传播路径），展示从源到汇的数据流

### 核心依赖
- `sarif_rust` (0.3.0)：提供 SARIF schema 类型和反序列化功能
- `serde_json` (1.0)：用于 SARIF 处理的 JSON 序列化/反序列化
- `async-openai` (0.33)：异步 OpenAI 兼容 API 客户端，用于 LLM 集成
- `tokio` (1)：异步运行时，支持 async/await
- `tree-sitter` (0.26)：多语言源码解析，支持 Python 和 Java
- `dotenv` (0.15)：加载 .env 环境变量
- `csv` (1.3)：CSV 文件读写，用于导出漏洞评审结果

`sarif_rust` 主要类型：
- `SarifLog`：根结构，包含一次或多次分析运行
- `Run`：单次工具执行及其结果
- `Result`：单个安全发现，包含位置、消息和代码流
- `CodeFlow`：污点分析路径，展示数据流

### AI 模块（`src/ai_chat.rs`）
- `get_a_client()`：创建配置好 MiniMax API 的异步 OpenAI 兼容客户端
- `chat_with_model()`：向 LLM 发送消息并返回响应

### SARIF 阅读器模块（`src/sarif_reader.rs`）
- `load_sarif_result()`：解析 SARIF 文件并打印前 3 条结果
- `TaintStep`：污点传播链路中的单个步骤（序号、message、文件路径、行号、源代码）
- `AuditResult`：AI 评审结果（结论、风险等级、修复建议、原始回复）
- `VulnerabilitySummary`：单个漏洞汇总信息（ruleId、描述、主位置、污点链路、AI评审结果）
- `build_vulnerability_summary()`：组合 SARIF 解析 + 源码行获取，返回所有漏洞的结构化汇总。**路径解析**：SARIF 文件所在目录的上一级作为源码根目录，URI 为相对路径拼接后读取
- `format_for_llm()`：将 `VulnerabilitySummary` 格式化为 LLM 可读的文本
- `export_to_csv()`：将漏洞汇总及 AI 评审结果导出为 CSV 文件（可用 Excel 打开）
- `remove_nulls()`：清理 JSON 输出中的 null 值

### 源码解析模块（`src/source_reader.rs`）
- `parse_source_string()`：解析内联代码字符串并打印 AST 结构
- `parse_source_file()`：异步解析源码文件，根据行号提取所在函数的最小作用域代码
- `get_source_line()`：根据行号从源文件中提取对应行的代码
- 支持 Python 和 Java 语言（通过 tree-sitter）

### 测试
测试位于 `src/lib.rs` 中的 `test_package` 模块：
- `test_parse_python`：测试 Python 源码字符串解析（AST 打印）
- `test_parse_pythonfile`：测试 Python 源码文件解析（异步，提取指定行号所在函数代码）
- `test_get_specific_line_code`：测试单行源代码获取

### GitHub Actions CI
`.github/workflows/rust.yml` 在每次 push 和 PR 到 master 分支时自动：
- 构建项目：`cargo build --verbose`
- 运行测试：`cargo test --verbose`

### 环境变量配置
项目使用 `.env` 文件配置 LLM API，需在项目根目录创建 `.env` 文件：
```
API_KEY=your_api_key_here
BASE_URL=your_api_base_url_here
MODEL_NAME=your_model_name_here
```

### 当前实现说明
- SARIF 文件路径目前在 `src/main.rs` 中硬编码（`SARIF_LOG` 常量）
- `load_sarif_result()` 默认处理并打印每个运行的前 3 个结果
- 新增 `build_vulnerability_summary()` + `format_for_llm()` 组合，可生成完整的漏洞汇总文本发送给 LLM
- `audit_vulnerability()` 逐条调用 LLM 评审，`export_to_csv()` 将结果导出为 CSV 文件留痕
- 异步操作使用 tokio 运行时和 `#[tokio::main]`
- LLM 集成使用 OpenAI 兼容接口连接 MiniMax API
