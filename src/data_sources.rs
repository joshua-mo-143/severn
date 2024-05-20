use crate::errors::Error;

#[async_trait::async_trait]
pub trait DataSource: Send + Sync {
    async fn retrieve_data(&self) -> Result<String, Error>;
}

#[cfg(feature = "qdrant")]
pub mod qdrant {
    use async_openai::types::Embedding;
    use qdrant_client::prelude::QdrantClient;
    use qdrant_client::qdrant::with_payload_selector::SelectorOptions;
    use qdrant_client::qdrant::{ScoredPoint, SearchPoints, WithPayloadSelector};
    use std::sync::atomic::AtomicI32;

    use crate::errors::Error;

    pub struct Qdrant {
        client: QdrantClient,
        collection_name: String,
        id: AtomicI32,
    }

    impl Qdrant {
        async fn search_embeddings(&self, embedding: Vec<f32>) -> Result<ScoredPoint, Error> {
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
                Some(res) => Ok(serde_json::to_string(res)),
                None => Err(Error::OptionIsNone),
            }
        }
    }
}
