use async_openai::error::OpenAIError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("LLM error: {0}")]
    LLMError(#[from] OpenAIError),
    #[error("serde_json error: {0}")]
    SerdeError(#[from] serde_json::error::Error),
    #[error("There's no agents in the pipeline!")]
    NoAgentsExist,
    #[error("Option expected to be Some but is None")]
    OptionIsNone,
    #[error("Searched data source but no results")]
    DataSourceNoMatch,
}
