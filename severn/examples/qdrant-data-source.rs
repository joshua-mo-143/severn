use severn::{
    agents::traits::Agent, data_sources::qdrant::Qdrant, files::CSVFile, pipeline::Pipeline,
    qdrant_client::client::QdrantClient,
};

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
        "You are an example agent from the Matrix, Neo.
         Your job is to serve as an example for all the other agents.

        Be concise in your answers."
            .to_string()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let example_agent = Arc::new(ExampleAgent::new());

    assert_eq!(example_agent.name(), String::from("Example agent"));

    let qdrant_client = QdrantClient::from_url("localhost:6334").build()?;
    let qdrant = Arc::new(Qdrant::new(qdrant_client));

    qdrant
        .embed_and_upsert::<CSVFile>("meme.csv".into())
        .await?;

    let pipeline_result = Pipeline::new()
        .add_data_source(qdrant)
        .add_agent(example_agent)
        .run_pipeline("Hello! This is a prompt.".to_owned())
        .await?;

    println!("{pipeline_result}");
    Ok(())
}
