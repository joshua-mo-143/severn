use async_openai::{config::OpenAIConfig, Client};
use severn::{agent::Agent, pipeline::Pipeline};
use std::sync::Arc;

struct ExampleAgent;

impl ExampleAgent {
    pub fn new() -> Self {
        Self
    }
}

impl Agent for ExampleAgent {
    fn name(&self) -> String {
        "Example agent".into()
    }

    fn system_message(&self) -> String {
        "You are an example agent, Neo. Your job is to serve as an example for all the other agents.".to_string()
    }
}

#[tokio::main]
async fn main() {
    let example_agent = Arc::new(ExampleAgent::new());

    assert_eq!(example_agent.name(), String::from("Example agent"));

    let pipeline = Pipeline::new().add_agent(example_agent);
}
