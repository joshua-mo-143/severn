use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    },
    Client,
};

use crate::errors::Error;

#[async_trait::async_trait]
pub trait Agent: Send + Sync {
    fn name(&self) -> String;
    fn system_message(&self) -> String;

    async fn prompt(
        &self,
        input: &str,
        data: String,
        client: Client<OpenAIConfig>,
    ) -> Result<String, Error> {
        let input = format!(
            "{input}

            Provided context:
            {}
            ",
            serde_json::to_string_pretty(&data)?
        );
        let res = client
            .chat()
            .create(
                CreateChatCompletionRequestArgs::default()
                    .model("gpt-4o")
                    .messages(vec![
                        //First we add the system message to define what the Agent does
                        ChatCompletionRequestMessage::System(
                            ChatCompletionRequestSystemMessageArgs::default()
                                .content(&self.system_message())
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
