use sarif_rust::SarifLog;
use serde_json::Value;
use std::error::Error;

use opensource_sast_verifier::ai::{get_a_client, chat_with_model};
use tokio;

const SARIF_LOG: &str = "D:/PythonProjects/Archery/outputs/total_results.sarif";

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

fn old_main() -> Result<(), Box<dyn Error>> {
    let log: SarifLog = sarif_rust::from_file(SARIF_LOG)?;

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


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    
    let client = get_a_client().await;
    let query = "你是哪个模型啊，可以简短的介绍一线自己吗。顺便告诉我你对rust编程知识的储备量。";
    
    match chat_with_model(client, query).await {
        Ok(res) => {
            if let Some(answer) = res {
                println!("模型的回答是：{}",answer);
            }
        },
        Err(e) => {
            eprint!("Something bad happend! {:?}", e);
        }
    }
    Ok(())
}