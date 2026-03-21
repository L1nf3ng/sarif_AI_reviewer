use std::error::Error;

use opensource_sast_verifier::ai::{get_a_client, chat_with_model};
use opensource_sast_verifier::sarif_reader::load_sarif_result;
use tokio;

const SARIF_LOG: &str = "D:/PythonProjects/Archery/outputs/total_results.sarif";


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    load_sarif_result(SARIF_LOG).await?;
    
    let client = get_a_client().await;
    let query = "你能验证codeql的漏洞报告吗，如果可以的话，你需要我告诉你哪些信息。";
    
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