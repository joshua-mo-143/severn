# Severn: Agent AI pipelines in Rust, made easy.
This crate aims to 

## Features
- Sequential pipelines for running AI agents and feeding the results into each other
- Exposes traits you can use to make your own traits and data sources

## How to use
### Agents
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

You can also use macros to define the implementation for your agent:

```rust
#[severn(name = "Example agent",
    system_message = "You are an example for how to write macros with agents"
)]
struct ExampleAgent;
```

### Data Sources
Severn also additionally exposes a `severn::data_sources::qdrant::Qdrant` struct for all of your RAG needs. `Qdrant` exposes a method for embedding and upserting single files into your Qdrant database. To use it, you need the `qdrant` feature enabled. This will also expose the `qdrant_client` crate as `severn::qdrant_client`.

Severn also additionally exposes a (WIP) `HttpClient` struct which allows you to add your own `reqwest::Client`. The `reqwest` crate is exposed as `severn::reqwest` - so you don't need to add the crate manually!

Need a custom data source? You can do exactly that! You only need to implement the `DataSource` trait - then you can add it to whatever pipeline you want.

### Text transformation
A `File` trait is exposed which the `Qdrant::embed_and_upsert` method takes. You can either use the `MarkdownFile` (or `CSVFile`) structs, or you can additionally create your own.

### Models
Currently only OpenAI is supported, but in the future support will be added for more.

## Contributions
Issues and PRs are welcome. However, unless the fix is very minor (for example a documentation typo), please make sure you open an issue first! This will avoid unnecessary work if it is either not in line with the overall vision of the crate(s) or warrants more attention than a single PR.

