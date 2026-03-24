# SAST Verifier

> 基于 Rust 的 SARIF 解析器 + SAST 漏洞验证工具，集成 LLM API 对 CodeQL / Semgrep 等静态分析工具的漏洞报告进行真伪验证、风险评级和修复建议。

## 背景

静态分析工具（CodeQL、Semgrep 等）产出的 SARIF 报告往往数量庞大，逐一人工复核成本极高。本工具将 SARIF 结果与源码上下文一同发送给 LLM，由 AI 辅助完成：

- 漏洞真伪鉴别（排除误报）
- 风险等级评估
- 修复方案建议

## 功能模块

### 1. SARIF 解析器 (`src/sarif_reader.rs`)

解析 SARIF 文件，结合源码生成结构化的漏洞汇总：

- **规则 ID**：漏洞类型标识
- **漏洞描述**：来自分析工具的说明
- **漏洞位置**：文件路径 + 行号
- **污点传播路径**：数据从 Source 到 Sink 的完整链路，每步附带源码行
- **AI 评审结果**：LLM 输出的真伪判断、风险等级、修复建议

### 2. LLM 集成 (`src/ai_chat.rs`)

通过 OpenAI 兼容接口连接 MiniMax API，将漏洞信息与源码上下文发送给 LLM：

- **角色**：白盒安全专家
- **输入**：污点传播链路 + 相关源码片段
- **输出**：漏洞真伪判断 + 风险等级 + 修复建议

### 3. 源码解析 (`src/source_reader.rs`)

使用 [tree-sitter](https://github.com/tree-sitter/tree-sitter) 对源码进行解析，根据行号获取对应源代码行：

- 支持 Python 和 Java
- 遍历语法树，提取节点类型、范围（行/列）、源码片段

## 架构

```
SARIF 文件（CodeQL / Semgrep 输出）
        │
        ▼
┌───────────────────────┐
│   sarif_reader.rs      │
│  解析 SARIF             │
│  拼接源码路径           │
└──────────┬────────────┘
           │ 漏洞信息 + URI
           ▼
┌───────────────────────┐   污点路径 + 源码
│   sarif_reader.rs      │ ───────────────────┐
│  build_vulnerability_  │                    │
│  summary()            │                    ▼
└──────────┬────────────┘            ┌───────────────┐
           │                          │  ai_chat.rs   │
           ▼                          │  LLM 验证     │
┌───────────────────────┐              └───────┬───────┘
│  source_reader.rs      │                      │
│  get_source_line()    │                      ▼
│  读取源码指定行        │            ┌─────────────────┐
└───────────────────────┘            │  export_to_csv() │
                                    │  导出 CSV 留痕   │
                                    └─────────────────┘
```

## 快速开始

### 1. 配置环境变量

在项目根目录创建 `.env` 文件：

```env
API_KEY=your-api-key
BASE_URL=https://api.minimaxi.com/v1
MODEL_NAME=MiniMax-M2
```

### 2. 修改 SARIF 文件路径

在 `src/main.rs` 中修改 `SARIF_LOG` 常量，指向你的 SARIF 文件：

```rust
const SARIF_LOG: &str = "path/to/your/results.sarif";
```

> 源码根目录由 SARIF 文件所在目录的上一级自动推导，无需额外配置。

### 3. 构建与运行

```bash
# 开发构建
cargo build
cargo run

# 发布构建（优化）
cargo build --release

# 仅检查编译错误
cargo check

# 运行测试
cargo test
```

## 输出结果

运行后生成以下产物：

- **控制台输出**：每条漏洞的 AI 评审结论
- **CSV 文件**（`vulnerability_audit.csv`）：完整的漏洞评审汇总表，包含规则ID、描述、主位置、污点传播链路、AI 评审结论、风险等级、修复建议，可直接用 Excel 打开

## 扩展方向

- [ ] 支持更多语言（JavaScript、Go、Rust 等）的 tree-sitter 解析器
- [ ] 批量处理多个 SARIF 文件
- [ ] 解析 LLM 回复结构化字段（风险等级、修复建议）并填入 Excel 对应列
