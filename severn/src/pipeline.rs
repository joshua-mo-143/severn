use crate::errors::Error;
use crate::models::PromptModel;
use crate::{agents::traits::Agent, data_sources::DataSource};
use std::sync::Arc;

pub struct Pipeline {
    agents: Vec<Arc<dyn Agent>>,
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl Pipeline {
    pub fn new() -> Self {
        Self { agents: Vec::new() }
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
        let mut context = String::from("None");

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

    pub async fn run_pipeline_with_initial_data<P: PromptModel, D: DataSource>(
        &self,
        prompt: String,
        model: P,
        data_source: D,
    ) -> Result<String, Error> {
        let mut context = data_source.retrieve_data().await?;

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

    pub async fn run_agent_at_index_with_initial_data<P: PromptModel, D: DataSource>(
        &self,
        prompt: String,
        index: usize,
        model: P,
        data_source: D,
    ) -> Result<String, Error> {
        let context = data_source.retrieve_data().await?;

        let agent = self.agents.get(index);

        match agent {
            Some(found_agent) => {
                let res = model.prompt(&prompt, context, found_agent).await?;

                Ok(res)
            }
            None => Err(Error::NoAgentsExist),
        }
    }

    pub async fn run_agent_by_name_with_initial_data<P: PromptModel, D: DataSource>(
        &self,
        prompt: String,
        name: &str,
        model: P,
        data_source: D,
    ) -> Result<String, Error> {
        let context = data_source.retrieve_data().await?;

        let agent = self.agents.iter().find(|x| x.name() == *name);

        match agent {
            Some(found_agent) => {
                let res = model.prompt(&prompt, context, found_agent).await?;

                Ok(res)
            }
            None => Err(Error::NoAgentsExist),
        }
    }

    pub fn remove_agent_at_index(mut self, index: usize) -> Self {
        self.agents.remove(index);

        self
    }

    pub fn remove_agent_by_name(mut self, name: String) -> Self {
        self.agents.retain(|x| x.name() == name);

        self
    }
}
