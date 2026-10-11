// stortinget.rs
use crate::error::ImportError;
use crate::stortinget_models::{CaseVotes, VoteProposals, VoteResults};
use reqwest::{Client, Url};
use serde::de::DeserializeOwned;

pub struct Fetched<T> {
    pub url: reqwest::Url,
    pub raw: bytes::Bytes,
    pub data: T,
}

#[derive(Clone)]
pub struct StortingetClient {
    http: Client,
    export_base: Url,
}

impl StortingetClient {
    pub fn new(http: Client, export_base: Url) -> Self {
        Self { http, export_base }
    }

    pub async fn votes_for_case(&self, case_id: i64) -> Result<Fetched<CaseVotes>, ImportError> {
        self.get("voteringer", "sakid", case_id).await
    }

    pub async fn results(&self, vote_id: i64) -> Result<Fetched<VoteResults>, ImportError> {
        self.get("voteringsresultat", "voteringid", vote_id).await
    }

    pub async fn proposals(&self, vote_id: i64) -> Result<Fetched<VoteProposals>, ImportError> {
        self.get("voteringsforslag", "voteringid", vote_id).await
    }

    async fn get<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        id_parameter: &str,
        id: i64,
    ) -> Result<Fetched<T>, ImportError> {
        // Endpoints are fixed relative paths, so joining only fails for a
        // base URL that cannot have paths, which is a startup configuration bug.
        let mut url = self
            .export_base
            .join(endpoint)
            .expect("export base URL must accept relative paths");

        url.query_pairs_mut()
            .append_pair(id_parameter, &id.to_string())
            .append_pair("format", "JSON");

        let raw = self
            .http
            .get(url.clone())
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;

        let data = serde_json::from_slice(&raw)?;

        Ok(Fetched { url, raw, data })
    }
}
