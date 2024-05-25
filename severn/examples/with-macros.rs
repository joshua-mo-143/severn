use severn::{agents::traits::Agent, pipeline::Pipeline};
use severn_macros::severn;
use std::sync::Arc;

#[severn(
    name = "Example agent",
    system_message = "This is an example of using macros with an agent"
)]
struct ExampleAgent;

impl ExampleAgent {
    pub fn new() -> Self {
        Self
    }
}

#[tokio::main]
async fn main() {
    let example_agent = Arc::new(ExampleAgent::new());

    println!("{}", example_agent.name());
    println!("{}", example_agent.system_message());

    assert_eq!(example_agent.name(), String::from("Example agent"));
    assert_eq!(
        example_agent.system_message(),
        String::from("This is an example of using macros with an agent")
    );

    let pipeline = Pipeline::new().add_agent(example_agent);
}
