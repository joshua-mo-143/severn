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
pub mod http {
    use crate::data_sources::DataSource;
    use crate::errors::Error;
    use qdrant_client::qdrant::Value;
    use reqwest::{header::HeaderMap, Client, Url};

    #[derive(Default)]
    pub struct HttpClientBuilder {
        pub http: Client,
        pub url: Option<Url>,
        pub headers: HeaderMap,
        pub body: Option<serde_json::Value>,
    }

    pub struct HttpClient {
        pub http: Client,
        url: Url,
        pub headers: HeaderMap,
        pub body: Option<serde_json::Value>,
    }

    impl HttpClientBuilder {
        pub fn new() -> Self {
            Self {
                http: Client::new(),
                ..Default::default()
            }
        }

        pub fn url(mut self, url: Url) -> Self {
            self.url = Some(url);

            self
        }

        pub fn add_header<K: reqwest::header::IntoHeaderName>(
            mut self,
            key: K,
            val: String,
        ) -> Self {
            self.headers.insert(key, val.parse().unwrap());

            self
        }

        pub fn set_headers(mut self, header_map: HeaderMap) -> Self {
            self.headers = header_map;

            self
        }

        pub fn body(mut self, body: serde_json::Value) -> Self {
            self.body = Some(body);

            self
        }

        pub fn build(self) -> anyhow::Result<HttpClient> {
            let Self {
                http,
                headers,
                url,
                body,
            } = self;

            let Some(url) = url else {
                return Err(anyhow::anyhow!("You need a URL!"));
            };

            let Some(body) = body else {
                return Err(anyhow::anyhow!("You can't have a GET request with a body!"));
            };

            Ok(HttpClient {
                http,
                url,
                body: Some(body),
                headers,
            })
        }
    }

    impl HttpClient {
        fn url(&self) -> Url {
            self.url.clone()
        }

        fn body(&self) -> serde_json::Value {
            self.body.clone().unwrap()
        }
    }

    #[async_trait::async_trait]
    impl DataSource for HttpClient {
        async fn retrieve_data(&self) -> Result<String, Error> {
            let client = reqwest::Client::new();
            let res = client
                .post(self.url())
                .json(&self.body())
                .send()
                .await
                .unwrap()
                .json::<Value>()
                .await
                .unwrap();

            Ok(serde_json::to_string_pretty(&res)?)
        }
    }
}
