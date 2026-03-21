use async_openai::{Client, config::OpenAIConfig};
use async_openai::types::chat::{
    CreateChatCompletionRequestArgs,
    ChatCompletionRequestUserMessageArgs,
    ChatCompletionRequestMessage,
};
use std::error::Error;

const API_KEY: &str = "sk-api-OhrNszaS2-4Y7GwxfKF0G-7mw-9Gf5tBamMvVKSeigenBZRMJ7S9h1t2P87si1MnKMwqvpl627oNAD1KJlfOznoUQuw1icjcUtKasaeWIUw6MI2Bqe00XYc";
const BASE_URL: &str = "https://api.minimaxi.com/v1";
const MODEL_NAME: &str = "MiniMax-M2.7";

pub async fn get_a_client() -> Client<OpenAIConfig>{
    let config = OpenAIConfig::new()
        .with_api_key(API_KEY)
        .with_api_base(BASE_URL);
    Client::with_config(config)
}


pub async fn chat_with_model(client: Client<OpenAIConfig>, message: &str ) -> Result<Option<String>,Box<dyn Error>>{
    let request = CreateChatCompletionRequestArgs::default()
        .model(MODEL_NAME)
        .messages(vec![ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessageArgs::default()
                .content(message)
                .build()?
        )])
        .max_tokens(1000u32)
        .build()?;

    let response = client.chat().create(request).await?;

    // 获取第一个选择的消息内容
    let content = response.choices.first()
        .and_then(|choice| choice.message.content.clone());

    Ok(content)
}

