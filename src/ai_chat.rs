use async_openai::{Client, config::OpenAIConfig};
use async_openai::types::chat::{
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs
};
use std::error::Error;

const API_KEY: &str = "sk-api-OhrNszaS2-4Y7GwxfKF0G-7mw-9Gf5tBamMvVKSeigenBZRMJ7S9h1t2P87si1MnKMwqvpl627oNAD1KJlfOznoUQuw1icjcUtKasaeWIUw6MI2Bqe00XYc";
const BASE_URL: &str = "https://api.minimaxi.com/v1";
const MODEL_NAME: &str = "MiniMax-M2.7";
const SYSTEM_PROMPT: &str  = "你是一个白盒专家，我通过codeql、semgrep等获得了一些sarif报告，但我没有时间去验证报告里漏洞的真伪。我会给你污点传播链路以及所涉及的相关源码，你需要帮我验证它们的真伪，同时评估漏洞的重要等级，并提供修复建议。";


pub async fn get_a_client() -> Client<OpenAIConfig>{
    let config = OpenAIConfig::new()
        .with_api_key(API_KEY)
        .with_api_base(BASE_URL);
    Client::with_config(config)
}


pub async fn chat_with_model(client: Client<OpenAIConfig>, message: &str ) -> Result<Option<String>,Box<dyn Error>>{
    let message_args = ChatCompletionRequestUserMessageArgs::default()
        .content(message)
        .build()?;
    let prompt_args = ChatCompletionRequestSystemMessageArgs::default()
        .content(SYSTEM_PROMPT)
        .build()?;
    
    let request = CreateChatCompletionRequestArgs::default()
        .model(MODEL_NAME)
        .messages(vec![
            ChatCompletionRequestMessage::System(prompt_args),
            ChatCompletionRequestMessage::User(message_args)
        ])
        .max_tokens(1000u32)
        .build()?;

    let response = client.chat().create(request).await?;

    // 获取第一个选择的消息内容
    let content = response.choices.first()
        .and_then(|choice| choice.message.content.clone());

    Ok(content)
}

