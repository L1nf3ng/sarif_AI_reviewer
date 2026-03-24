use sarif_rust::SarifLog;
use serde_json::Value;
use std::error::Error;

use crate::source_reader::get_source_line;


/// 污点传播链路中的单个步骤
#[derive(Debug)]
pub struct TaintStep {
    /// 步骤序号
    pub step_number: usize,
    /// 该步骤的 message.text
    pub message: String,
    /// 源代码文件路径
    pub file_path: String,
    /// 行号（1-based）
    pub line_number: usize,
    /// 该行源代码
    pub source_code: String,
}

/// 单个漏洞的汇总信息
#[derive(Debug)]
pub struct VulnerabilitySummary {
    /// 规则 ID
    pub rule_id: String,
    /// 漏洞描述信息
    pub message: String,
    /// 漏洞主位置文件路径
    pub file_path: String,
    /// 漏洞主位置行号
    pub line_number: usize,
    /// 污点传播链路
    pub taint_chain: Vec<TaintStep>,
}


/// 递归删除JSON中的null值, AI写的很不错，值得学习。
fn remove_nulls(value: &mut Value) {
    match value {
        Value::Object(map) => {
            // 收集需要删除的键
            let keys_to_remove: Vec<String> = map
                .iter()
                .filter(|(_, v)| v.is_null())
                .map(|(k, _)| k.clone())
                .collect();

            // 删除 null 值的键
            for key in keys_to_remove {
                map.remove(&key);
            }

            // 递归处理剩余的值
            for (_, v) in map.iter_mut() {
                remove_nulls(v);
            }
        }
        Value::Array(arr) => {
            // 递归处理数组中的每个元素
            for item in arr.iter_mut() {
                remove_nulls(item);
            }
        }
        _ => {}
    }
}

pub async fn load_sarif_result(filename: &str) -> Result<(), Box<dyn Error>> {
    let log: SarifLog = sarif_rust::from_file(filename)?;

    for run in &log.runs {
        println!("Tool: {}", run.tool.driver.name);

        if let Some(results) = &run.results {
            for (id, result) in results.iter().enumerate() {
                if id == 3 {
                    break;
                }
                // 将 result 序列化为 JSON
                let mut json_value = serde_json::to_value(result)?;
                remove_nulls(&mut json_value);

                // 格式化输出 JSON
                // 观察输出，我们需要的数据有：
                // ruleid
                // message
                // --> codeFlows[]
                // --  --> threadFlows []
                // --  --  --> locations [
                // --  --  --  --> location(message, physicalLocation { artifactLocation(url), region(end_column, start_column, startLine)}]
                if let Some(vuln_info) = json_value.as_object() {
                    let vuln_type = &vuln_info["ruleId"];
                    let vuln_detail = vuln_info["message"]["text"].as_str().unwrap_or("N/A");
                    println!("漏洞名称：{} 描述信息：{}", vuln_type, vuln_detail);

                    // 提取 codeFlows（污点传播路径）
                    if let Some(code_flows) = vuln_info["codeFlows"].as_array() {
                        for (cf_idx, code_flow) in code_flows.iter().enumerate() {
                            println!("  CodeFlow #{}", cf_idx + 1);
                            if let Some(thread_flows) = code_flow["threadFlows"].as_array() {
                                for (tf_idx, thread_flow) in thread_flows.iter().enumerate() {
                                    println!("    ThreadFlow #{}", tf_idx + 1);
                                    if let Some(locations) = thread_flow["locations"].as_array() {
                                        for (loc_idx, loc) in locations.iter().enumerate() {
                                            let loc_msg = loc["location"]["message"]["text"].as_str().unwrap_or("N/A");
                                            let uri = loc["location"]["physicalLocation"]["artifactLocation"]["uri"].as_str().unwrap_or("N/A");
                                            let start_line = loc["location"]["physicalLocation"]["region"]["startLine"].as_i64().unwrap_or(0);
                                            let start_col = loc["location"]["physicalLocation"]["region"]["startColumn"].as_i64().unwrap_or(0);
                                            let _end_col = loc["location"]["physicalLocation"]["region"]["endColumn"].as_i64().unwrap_or(0);
                                            println!("      Step {}: {} 位于 {}:{}:{}  →  {}", loc_idx + 1, loc_msg, uri, start_line, start_col, _end_col);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // 提取主漏洞位置
                    if let Some(locations) = vuln_info["locations"].as_array() {
                        for loc in locations {
                            let uri = loc["physicalLocation"]["artifactLocation"]["uri"].as_str().unwrap_or("N/A");
                            let start_line = loc["physicalLocation"]["region"]["startLine"].as_i64().unwrap_or(0);
                            let start_col = loc["physicalLocation"]["region"]["startColumn"].as_i64().unwrap_or(0);
                            let _end_col = loc["physicalLocation"]["region"]["endColumn"].as_i64().unwrap_or(0);
                            println!("  漏洞位置: {}:{}:{}", uri, start_line, start_col);
                        }
                    }

                }

                println!("{}", "=".repeat(80)); // 分隔线
            }
        }
    }

    Ok(())
}

/// 根据 SARIF 文件生成漏洞汇总信息，结合源代码行
pub async fn build_vulnerability_summary(
    sarif_path: &str,
    language: &str,
) -> Result<Vec<VulnerabilitySummary>, Box<dyn Error>> {
    let log: SarifLog = sarif_rust::from_file(sarif_path)?;

    // 源码根目录：SARIF 文件所在目录的上一级
    let sarif_dir = std::path::Path::new(sarif_path)
        .parent()
        .ok_or("无法获取 SARIF 文件所在目录")?;
    let source_root = sarif_dir
        .parent()
        .ok_or("无法获取源码根目录")?;

    let mut summaries = Vec::new();

    for run in &log.runs {
        if let Some(results) = &run.results {
            for result in results {
                let mut json_value = serde_json::to_value(result)?;
                remove_nulls(&mut json_value);

                if let Some(vuln_info) = json_value.as_object() {
                    let rule_id = vuln_info
                        .get("ruleId")
                        .and_then(|v| v.as_str())
                        .unwrap_or("N/A")
                        .to_string();
                    let message = vuln_info
                        .get("message")
                        .and_then(|v| v.get("text"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("N/A")
                        .to_string();

                    // 提取主漏洞位置
                    let (file_path, line_number) = vuln_info
                        .get("locations")
                        .and_then(|v| v.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|loc| {
                            let uri = loc
                                .get("physicalLocation")
                                .and_then(|v| v.get("artifactLocation"))
                                .and_then(|v| v.get("uri"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("N/A");
                            let line = loc
                                .get("physicalLocation")
                                .and_then(|v| v.get("region"))
                                .and_then(|v| v.get("startLine"))
                                .and_then(|v| v.as_i64())
                                .unwrap_or(0) as usize;
                            Some((uri.to_string(), line))
                        })
                        .unwrap_or_else(|| ("N/A".to_string(), 0));

                    // 提取污点传播链路
                    let mut taint_chain = Vec::new();
                    let mut step_number = 1;

                    if let Some(code_flows) = vuln_info.get("codeFlows").and_then(|v| v.as_array()) {
                        for code_flow in code_flows {
                            if let Some(thread_flows) = code_flow.get("threadFlows").and_then(|v| v.as_array()) {
                                for thread_flow in thread_flows {
                                    if let Some(locations) = thread_flow.get("locations").and_then(|v| v.as_array()) {
                                        for loc in locations {
                                            let loc_msg = loc
                                                .get("location")
                                                .and_then(|v| v.get("message"))
                                                .and_then(|v| v.get("text"))
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("N/A")
                                                .to_string();
                                            let uri = loc
                                                .get("location")
                                                .and_then(|v| v.get("physicalLocation"))
                                                .and_then(|v| v.get("artifactLocation"))
                                                .and_then(|v| v.get("uri"))
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("N/A");
                                            let line = loc
                                                .get("location")
                                                .and_then(|v| v.get("physicalLocation"))
                                                .and_then(|v| v.get("region"))
                                                .and_then(|v| v.get("startLine"))
                                                .and_then(|v| v.as_i64())
                                                .unwrap_or(0) as usize;

                                            // 拼接源码绝对路径（源码根目录 + SARIF 中的相对 URI）
                                            let absolute_path = source_root.join(uri);
                                            let absolute_path_str =
                                                absolute_path.to_str().ok_or("路径转换失败")?;

                                            // 获取源代码行
                                            let source_code = match get_source_line(absolute_path_str, language, line).await {
                                                Ok(code) => code,
                                                Err(_) => format!("<无法读取行 {}: {}>", line, absolute_path_str),
                                            };

                                            taint_chain.push(TaintStep {
                                                step_number,
                                                message: loc_msg,
                                                file_path: uri.to_string(),
                                                line_number: line,
                                                source_code,
                                            });
                                            step_number += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    summaries.push(VulnerabilitySummary {
                        rule_id,
                        message,
                        file_path,
                        line_number,
                        taint_chain,
                    });
                }
            }
        }
    }

    Ok(summaries)
}

/// 将漏洞汇总格式化为 LLM 可读的文本
pub fn format_for_llm(summaries: &[VulnerabilitySummary]) -> String {
    let mut output = String::new();

    for (idx, summary) in summaries.iter().enumerate() {
        output.push_str(&format!("## 漏洞 #{}\n", idx + 1));
        output.push_str(&format!("规则 ID: {}\n", summary.rule_id));
        output.push_str(&format!("描述: {}\n", summary.message));
        output.push_str(&format!(
            "主位置: {}:{}\n",
            summary.file_path, summary.line_number
        ));

        if !summary.taint_chain.is_empty() {
            output.push_str("\n污点传播链路:\n");
            for step in &summary.taint_chain {
                output.push_str(&format!(
                    "  Step {}: {} → 位于 {}:{}\n",
                    step.step_number, step.message, step.file_path, step.line_number
                ));
                output.push_str(&format!("    代码: {}\n", step.source_code));
            }
        }

        output.push_str("\n");
    }

    output
}