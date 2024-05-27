use std::sync::Arc;

use crate::agents::traits::Agent;
use anyhow::Result;
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    },
    Client,
};
use async_trait::async_trait;

use crate::errors::Error;

pub struct OpenAI {
    client: Client<OpenAIConfig>,
}

impl OpenAI {
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("OPENAI_API_KEY")?;
        let config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_org_id("severn");

        let client = Client::with_config(config);

        Ok(Self { client })
    }
}

#[async_trait]
pub trait PromptModel {
    async fn prompt(
        &self,
        prompt: &str,
        data: String,
        agent: &Arc<dyn Agent>,
    ) -> Result<String, Error>;
}

#[async_trait]
impl PromptModel for OpenAI {
    async fn prompt(
        &self,
        prompt: &str,
        data: String,
        agent: &Arc<dyn Agent>,
    ) -> Result<String, Error> {
        let input = format!(
            "{prompt}

            Provided context:
            {}
            ",
            serde_json::to_string_pretty(&data)?
        );
        let res = self
            .client
            .chat()
            .create(
                CreateChatCompletionRequestArgs::default()
                    .model("gpt-4o")
                    .messages(vec![
                        //First we add the system message to define what the Agent does
                        ChatCompletionRequestMessage::System(
                            ChatCompletionRequestSystemMessageArgs::default()
                                .content(&agent.system_message())
                                .build()?,
                        ),
                        //Then we add our prompt
                        ChatCompletionRequestMessage::User(
                            ChatCompletionRequestUserMessageArgs::default()
                                .content(input)
                                .build()?,
                        ),
                    ])
                    .build()?,
            )
            .await
            .map(|res| {
                //We extract the first one
                match res.choices[0].message.content.clone() {
                    Some(res) => Ok(res),
                    None => Err(Error::OptionIsNone),
                }
            })??;

        println!("Retrieved result from prompt: {res}");

        Ok(res)
    }
}
