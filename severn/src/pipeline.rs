use crate::errors::Error;
use crate::models::PromptModel;
use crate::{agents::traits::Agent, data_sources::DataSource};
use std::sync::Arc;

pub struct Pipeline {
    agents: Vec<Arc<dyn Agent>>,
    data_source: Option<Arc<dyn DataSource>>,
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            agents: Vec::new(),
            data_source: None,
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

    pub async fn run_pipeline<P: PromptModel>(
        &self,
        prompt: String,
        model: P,
    ) -> Result<String, Error> {
        let mut context = match &self.data_source {
            Some(source) => source.retrieve_data().await?,
            None => String::new(),
        };
        let mut agents = self.agents.iter().peekable();

        if agents.len() == 0 {
            return Err(Error::NoAgentsExist);
        }

        while let Some(agent) = agents.next() {
            let res = model.prompt(&prompt, context.to_owned(), agent).await?;

            context = res.clone();

            if agents.peek().is_none() {
                return Ok(res);
            }
        }

        Err(Error::NoAgentsExist)
    }

    pub async fn run_agent_at_index<P: PromptModel>(
        &self,
        prompt: String,
        index: usize,
        model: P,
    ) -> Result<String, Error> {
        let context = match &self.data_source {
            Some(source) => source.retrieve_data().await?,
            None => String::new(),
        };

        let agent = self.agents.get(index);

        match agent {
            Some(found_agent) => {
                let res = model
                    .prompt(&prompt, context.to_owned(), found_agent)
                    .await?;

                Ok(res)
            }
            None => Err(Error::NoAgentsExist),
        }
    }

    pub async fn run_agent_by_name<P: PromptModel>(
        &self,
        prompt: String,
        name: &str,
        model: P,
    ) -> Result<String, Error> {
        let context = match &self.data_source {
            Some(source) => source.retrieve_data().await?,
            None => String::new(),
        };

        let agent = self.agents.iter().find(|x| x.name() == *name);

        match agent {
            Some(found_agent) => {
                let res = model
                    .prompt(&prompt, context.to_owned(), found_agent)
                    .await?;

                Ok(res)
            }
            None => Err(Error::NoAgentsExist),
        }
    }
}
