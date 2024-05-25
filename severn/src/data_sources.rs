use crate::errors::Error;

#[async_trait::async_trait]
pub trait DataSource: Send + Sync {
    async fn retrieve_data(&self) -> Result<String, Error>;
}

#[cfg(feature = "qdrant")]
pub mod qdrant {
    use crate::data_sources::DataSource;
    use async_openai::config::OpenAIConfig;
    use async_openai::types::{CreateEmbeddingRequest, EmbeddingInput};
    use async_openai::{Client, Embeddings};
    use qdrant_client::client::Payload;
    use qdrant_client::prelude::QdrantClient;
    use qdrant_client::qdrant::with_payload_selector::SelectorOptions;
    use qdrant_client::qdrant::PointStruct;
    use qdrant_client::qdrant::{ScoredPoint, SearchPoints, WithPayloadSelector};

    use std::path::PathBuf;

    use crate::errors::Error;
    use crate::files::File;

    pub struct Qdrant {
        client: QdrantClient,
        collection_name: String,
        openai_client: Client<OpenAIConfig>,
    }

    impl Qdrant {
        pub fn new(client: QdrantClient) -> Self {
            let api_key = std::env::var("OPENAI_API_KEY").expect("No OPENAI_API_KEY");
            let config = OpenAIConfig::new()
                .with_api_key(api_key)
                .with_org_id("severn");

            let openai_client = Client::with_config(config);

            Self {
                client,
                collection_name: "Meme".to_string(),
                openai_client,
            }
        }

        pub async fn embed_and_upsert<T: File>(&self, path: PathBuf) -> anyhow::Result<()> {
            let file = T::from_filepath(path)?;

            let embeddings = self.embed_file(file.parse()).await?;

            for embedding in embeddings {
                self.upsert_embedding(
                    embedding,
                    serde_json::json!({
                        "document": file.contents()
                    })
                    .try_into()
                    .unwrap(),
                )
                .await
                .unwrap();
            }

            Ok(())
        }

        pub async fn embed_file(
            &self,
            chunked_contents: Vec<String>,
        ) -> anyhow::Result<Vec<Vec<f32>>> {
            let embedding_request = CreateEmbeddingRequest {
                model: "text-embedding-ada-002".to_string(),
                input: EmbeddingInput::StringArray(chunked_contents.to_owned()),
                encoding_format: None, // defaults to f32
                user: None,
                dimensions: Some(1536),
            };

            let embeddings = Embeddings::new(&self.openai_client)
                .create(embedding_request)
                .await?;

            if embeddings.data.is_empty() {
                return Err(anyhow::anyhow!(
                    "There were no embeddings returned by OpenAI!"
                ));
            }

            Ok(embeddings.data.into_iter().map(|x| x.embedding).collect())
        }

        pub async fn embed_sentence(&self, prompt: &str) -> anyhow::Result<Vec<f32>> {
            let embedding_request = CreateEmbeddingRequest {
                model: "text-embedding-ada-002".to_string(),
                input: EmbeddingInput::String(prompt.to_owned()),
                encoding_format: None, // defaults to f32
                user: None,
                dimensions: Some(1536),
            };

            let embeddings = Embeddings::new(&self.openai_client)
                .create(embedding_request)
                .await?;

            let embedding = embeddings.data.first();

            match embedding {
                Some(res) => Ok(res.embedding.clone()),
                None => Err(anyhow::anyhow!(
                    "There were no embeddings returned by OpenAI!"
                )),
            }
        }

        pub async fn upsert_embedding(
            &self,
            embedding: Vec<f32>,
            payload: Payload,
        ) -> anyhow::Result<()> {
            let points = vec![PointStruct::new(
                uuid::Uuid::new_v4().to_string(),
                embedding,
                payload,
            )];
            self.client
                .upsert_points(self.collection_name.to_owned(), None, points, None)
                .await?;

            Ok(())
        }

        pub async fn search_embeddings(&self, embedding: Vec<f32>) -> Result<ScoredPoint, Error> {
            let payload_selector = WithPayloadSelector {
                selector_options: Some(SelectorOptions::Enable(true)),
            };

            let search_points = SearchPoints {
                collection_name: self.collection_name.to_owned(),
                vector: embedding,
                limit: 1,
                with_payload: Some(payload_selector),
                ..Default::default()
            };

            let search_result = self
                .client
                .search_points(&search_points)
                .await
                .inspect_err(|x| println!("An error occurred while searching for points: {x}"))
                .unwrap();

            let result = search_result.result.into_iter().next();

            match result {
                Some(res) => Ok(res),
                None => Err(Error::DataSourceNoMatch),
            }
        }
    }

    #[async_trait::async_trait]
    impl DataSource for Qdrant {
        async fn retrieve_data(&self) -> Result<String, Error> {
            let vec: Vec<f32> = Vec::new();
            let embedding = self.search_embeddings(vec).await?;

            let payload_field_value = embedding.payload.get("document");

            match payload_field_value {
                Some(res) => Ok(serde_json::to_string(res).unwrap()),
                None => Err(Error::OptionIsNone),
            }
        }
    }
}

#[cfg(feature = "http")]
mod http {
    use reqwest::{Client, IntoUrl};

    pub struct HttpClientBuilder {
        http: Client,
        url: Option<Url>,
        request_method: RequestMethod,
        body: Option<serde_json::Value>,
    }

    pub struct HttpClient {
        http: Client,
        url: Url,
        request_method: RequestMethod,
        body: Option<serde_json::Value>,
    }

    pub enum RequestMethod {
        Get,
        Post,
    }

    impl HttpClientBuilder {
        pub fn new() -> Self {
            Self {
                http: Client::new(),
                request_method: RequestMethod::Get,
                ..Default::default()
            }
        }

        pub fn url(mut self, url: IntoUrl) -> Self {
            self.url = Some(url);

            self
        }

        pub fn request_method(mut self, request_method: RequestMethod) -> Self {
            self.url = Some(request_method);

            self
        }

        pub fn body(mut self, body: serde_json::Value) -> Self {
            self.body = Some(body);

            self
        }

        pub fn build(self) -> anyhow::Result<HttpClient> {
            let (http, url, request_method, body) = self;

            let Some(url) = url else {
                return Err(anyhow::anyhow!("You need a URL!"));
            };

            if request_method == RequestMethod::Get && body.is_some() {
                return Err(anyhow::anyhow!("You can't have a GET request with a body!"));
            }

            if request_method == RequestMethod::Post && !body.is_some() {
                return Err(anyhow::anyhow!(
                    "You didn't set a body on your POST request!"
                ));
            }

            Ok(HttpClient {
                http,
                url,
                request_method,
                body,
            })
        }
    }

    impl DataSource for HttpClient {
        async fn retrieve_data(&self) -> Result<String, Error> {
            let res = self
                .client
                .post(self.url)
                .json(self.body)
                .send()
                .await
                .unwrap()
                .json::<Value>()
                .unwrap();

            Ok(serde_json::to_string_pretty(res))
        }
    }
}
