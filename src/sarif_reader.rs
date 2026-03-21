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
                let json_output = serde_json::to_string_pretty(&json_value)?;
                println!("{}", json_output);
                println!("{}", "=".repeat(80)); // 分隔线
            }
        }
    }

    Ok(())
}