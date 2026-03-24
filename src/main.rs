use std::error::Error;

use opensource_sast_verifier::ai_chat::{get_a_client, chat_with_model};
use opensource_sast_verifier::sarif_reader::{build_vulnerability_summary, format_for_llm, load_sarif_result};
use tokio;

const SARIF_LOG: &str = "D:/PythonProjects/Archery/outputs/total_results.sarif";


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    load_sarif_result(SARIF_LOG).await?;

    // 演示：构建漏洞汇总文本并发送给 LLM
    let summaries = build_vulnerability_summary(SARIF_LOG, "python").await?;
    let formatted = format_for_llm(&summaries);
    println!("漏洞汇总文本：\n{}", formatted);

    let client = get_a_client().await;
    match chat_with_model(client, &formatted).await {
        Ok(res) => {
            if let Some(answer) = res {
                println!("模型的回答是：{}", answer);
            }
        },
        Err(e) => {
            eprint!("Something bad happend! {:?}", e);
        }
    }

    Ok(())
}