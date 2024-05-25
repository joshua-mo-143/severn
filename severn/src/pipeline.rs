use crate::errors::Error;
use crate::{agents::traits::Agent, data_sources::DataSource};
use async_openai::{config::OpenAIConfig, Client};
use std::sync::Arc;

pub struct Pipeline {
    agents: Vec<Arc<dyn Agent>>,
    data_source: Option<Arc<dyn DataSource>>,
    client: Client<OpenAIConfig>,
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl Pipeline {
    pub fn new() -> Self {
        let api_key = std::env::var("OPENAI_API_KEY").expect("No OPENAI_API_KEY");
        let config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_org_id("severn");

        let client = Client::with_config(config);
        Self {
            agents: Vec::new(),
            data_source: None,
            client,
        }
    }

    pub fn add_data_source(mut self, data_source: Arc<dyn DataSource>) -> Self {
        self.data_source = Some(data_source);

        self
    }

    pub fn add_agent(mut self, agent: Arc<dyn Agent>) -> Self {
        self.agents.push(agent);

        self
    }

    pub async fn run_pipeline(&self, prompt: String) -> Result<String, Error> {
        let mut context = match &self.data_source {
            Some(source) => source.retrieve_data().await?,
            None => String::new(),
        };
        let mut agents = self.agents.iter().peekable();

        if agents.len() == 0 {
            return Err(Error::NoAgentsExist);
        }

        while let Some(agent) = agents.next() {
            let res = agent
                .prompt(&prompt, context.to_owned(), self.client.to_owned())
                .await?;
            context = res.clone();

            if agents.peek().is_none() {
                return Ok(res);
            }
        }

        Err(Error::NoAgentsExist)
    }
}
