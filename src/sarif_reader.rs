use sarif_rust::SarifLog;
use serde_json::Value;
use std::error::Error;


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