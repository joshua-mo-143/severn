# Severn: Agent AI pipelines in Rust, made easy.
Do you hate the fact that the llm-chain crate is too macro heavy? Me too. Let's make a new crate that doesn't suck.

## How to use
First, you need to define a struct that will serve as your agent:

``` rust
use severn::agent::Agent;

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
```

Next, you can now use your agent by turning it into an `Arc<T>` and adding your agent to the pipeline:

``` rust
use std::sync::Arc;
use severn::pipeline::Pipeline;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let example_agent = Arc::new(ExampleAgent::new());

    assert_eq!(example_agent.name(), String::from("Example agent"));

    let pipeline = Pipeline::new().add_agent(example_agent);
    
    let prompt = "What is your job, Neo?".to_string();
    
    let result = pipeline.run_pipeline(prompt).await?;
    
    println!("{result}");
}
```
