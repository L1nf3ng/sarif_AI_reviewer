# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based SARIF (Static Analysis Results Interchange Format) parser and verifier for SAST (Static Application Security Testing) results. The tool processes security scan outputs from tools like CodeQL, Semgrep, and other static analyzers that export to SARIF format.

## Build and Run Commands

```bash
# Build the project
cargo build

# Run the project
cargo run

# Build in release mode (optimized)
cargo build --release

# Check for compilation errors without building
cargo check

# Run tests (when added)
cargo test
```

## Architecture

### Core Data Flow
The application follows a simple pipeline:
1. **SARIF Parsing**: Uses `sarif_rust` crate to deserialize SARIF files into Rust structs
2. **Result Extraction**: Iterates through runs (scan executions) and their results (findings)
3. **Information Extraction**: For each security finding, extracts:
   - Problem type and message
   - Rule identifier and location
   - Code flows (taint propagation paths) showing data flow from source to sink

### Key Dependencies
- `sarif_rust` (0.3.0): Provides SARIF schema types and deserialization. The main types used:
  - `SarifLog`: Root structure containing one or more analysis runs
  - `Run`: A single tool execution with results
  - `Result`: Individual security finding with location, message, and code flows
  - `CodeFlow`: Taint analysis path showing data flow

### Current Implementation Notes
- The SARIF file path is currently hardcoded in `src/main.rs` (`SARIF_LOG` constant)
- The tool processes and prints the first 3 results from each run
- Output focuses on: message, rule ID, location, and taint flow steps