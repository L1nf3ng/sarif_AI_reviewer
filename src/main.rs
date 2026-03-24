use async_openai::Client;
use async_openai::config::OpenAIConfig;
use std::error::Error;

use opensource_sast_verifier::ai_chat::{get_a_client, chat_with_model};
use opensource_sast_verifier::sarif_reader::{build_vulnerability_summary, export_to_csv, format_for_llm, load_sarif_result, AuditResult, VulnerabilitySummary};
use tokio;

const SARIF_LOG: &str = "D:/PythonProjects/Archery/outputs/total_results.sarif";
const CSV_OUTPUT: &str = "D:/PythonProjects/Archery/outputs/vulnerability_audit.csv";

/// 对单条漏洞调用 LLM 评审
async fn audit_vulnerability(
    client: &Client<OpenAIConfig>,
    summary: &VulnerabilitySummary,
) -> AuditResult {
    let formatted = format_for_llm(&[summary.clone()]);
    match chat_with_model(client.clone(), &formatted).await {
        Ok(Some(response)) => {
            println!("漏洞评审结果：{}", response);
            AuditResult {
                verdict: Some(response.clone()),
                severity: None,
                fix_suggestion: None,
                raw_response: Some(response),
            }
        }
        Ok(None) => AuditResult::default(),
        Err(e) => {
            eprintln!("LLM 调用失败: {:?}", e);
            AuditResult::default()
        }
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    load_sarif_result(SARIF_LOG).await?;

    // 构建漏洞汇总
    let mut summaries = build_vulnerability_summary(SARIF_LOG, "python").await?;
    println!("共提取 {} 条漏洞，开始 AI 评审...\n", summaries.len());

    // 逐条调用 LLM 评审
    let client = get_a_client().await;
    for summary in &mut summaries {
        println!("评审漏洞: {} @ {}:{}",
            summary.rule_id, summary.file_path, summary.line_number);
        summary.audit_result = audit_vulnerability(&client, summary).await;
        println!("{}\n", "=".repeat(60));
    }

    // 导出 CSV
    export_to_csv(&summaries, CSV_OUTPUT)?;

    Ok(())
}