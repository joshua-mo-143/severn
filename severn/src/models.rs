use std::sync::Arc;

use crate::agents::traits::Agent;
use anyhow::Result;
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
        CreateEmbeddingRequest, EmbeddingInput,
    },
    Client, Embeddings,
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

    pub fn from_env_with_org_id(org_id: &str) -> Result<Self> {
        let api_key = std::env::var("OPENAI_API_KEY")?;
        let config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_org_id(org_id);

        let client = Client::with_config(config);

        Ok(Self { client })
    }

    pub fn from_str(api_key: &str) -> Result<Self> {
        let config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_org_id("severn");

        let client = Client::with_config(config);

        Ok(Self { client })
    }

    pub fn from_str_with_org_id(api_key: &str, org_id: &str) -> Result<Self> {
        let config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_org_id(org_id);

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
pub trait EmbedModel {
    async fn embed_file(&self, chunked_contents: Vec<String>) -> anyhow::Result<Vec<Vec<f32>>>;
    async fn embed_sentence(&self, prompt: &str) -> anyhow::Result<Vec<f32>>;
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

#[async_trait]
impl EmbedModel for OpenAI {
    async fn embed_file(&self, chunked_contents: Vec<String>) -> anyhow::Result<Vec<Vec<f32>>> {
        let embedding_request = CreateEmbeddingRequest {
            model: "text-embedding-ada-002".to_string(),
            input: EmbeddingInput::StringArray(chunked_contents.to_owned()),
            encoding_format: None, // defaults to f32
            user: None,
            dimensions: Some(1536),
        };

        let embeddings = Embeddings::new(&self.client)
            .create(embedding_request)
            .await?;

        if embeddings.data.is_empty() {
            return Err(anyhow::anyhow!(
                "There were no embeddings returned by OpenAI!"
            ));
        }

        Ok(embeddings.data.into_iter().map(|x| x.embedding).collect())
    }

    async fn embed_sentence(&self, prompt: &str) -> anyhow::Result<Vec<f32>> {
        let embedding_request = CreateEmbeddingRequest {
            model: "text-embedding-ada-002".to_string(),
            input: EmbeddingInput::String(prompt.to_owned()),
            encoding_format: None, // defaults to f32
            user: None,
            dimensions: Some(1536),
        };

        let embeddings = Embeddings::new(&self.client)
            .create(embedding_request)
            .await?;

        let embedding = embeddings.data.into_iter().next();

        match embedding {
            Some(res) => Ok(res.embedding),
            None => Err(anyhow::anyhow!(
                "There were no embeddings returned by OpenAI!"
            )),
        }
    }
}
