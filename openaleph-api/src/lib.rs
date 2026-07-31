mod models;

pub use models::*;

use reqwest::header::{AUTHORIZATION, USER_AGENT};

#[derive(Clone, Debug)]
pub struct Client {
    http: reqwest::Client,
    base_url: String,
    authorization: String,
    user_agent: String,
}

impl Client {
    pub fn new(
        base_url: impl Into<String>,
        token: impl AsRef<str>,
        user_agent: impl Into<String>,
    ) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            authorization: format!("Bearer {}", token.as_ref()),
            user_agent: user_agent.into(),
        }
    }

    pub async fn status(&self) -> reqwest::Result<Status> {
        self.get("status").await
    }

    pub async fn metadata(&self) -> reqwest::Result<Metadata> {
        self.get("metadata").await
    }

    pub async fn status_and_metadata(&self) -> reqwest::Result<(Status, Metadata)> {
        let status = self.status().await?;
        let metadata = self.metadata().await?;
        Ok((status, metadata))
    }

    pub async fn search(&self, query: &str, limit: usize) -> reqwest::Result<SearchResponse> {
        self.http
            .get(format!("{}/api/2/entities", self.base_url))
            .header(AUTHORIZATION, &self.authorization)
            .header(USER_AGENT, &self.user_agent)
            .query(&[
                ("q", query.to_owned()),
                ("limit", limit.to_string()),
                ("highlight", "true".to_owned()),
                ("dehydrate", "true".to_owned()),
            ])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    pub async fn investigations(
        &self,
        query: &str,
        limit: usize,
    ) -> reqwest::Result<InvestigationsResponse> {
        self.http
            .get(format!("{}/api/2/collections", self.base_url))
            .header(AUTHORIZATION, &self.authorization)
            .header(USER_AGENT, &self.user_agent)
            .query(&[
                ("q", query.to_owned()),
                ("limit", limit.to_string()),
                ("filter:category", "casefile".to_owned()),
            ])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    async fn get<T>(&self, endpoint: &str) -> reqwest::Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.http
            .get(format!("{}/api/2/{}", self.base_url, endpoint))
            .header(AUTHORIZATION, &self.authorization)
            .header(USER_AGENT, &self.user_agent)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{
        matchers::{header, method, path, query_param},
        Mock, MockServer, ResponseTemplate,
    };

    #[tokio::test]
    async fn fetches_status_and_metadata() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/2/status"))
            .and(header("authorization", "Bearer test-token"))
            .and(header("user-agent", "openaleph-tui/0.5.0-test"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [],
                "total": 0
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/api/2/metadata"))
            .and(header("authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "ok",
                "maintenance": false,
                "app": {
                    "title": "Test Aleph",
                    "version": "1.0.0",
                    "ftm_version": "4.0.0"
                }
            })))
            .mount(&server)
            .await;

        let client = Client::new(server.uri(), "test-token", "openaleph-tui/0.5.0-test");
        let (status, metadata) = client.status_and_metadata().await.unwrap();

        assert_eq!(status.total, 0);
        assert_eq!(metadata.status, "ok");
    }

    #[tokio::test]
    async fn returns_http_errors() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/2/status"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&server)
            .await;

        let error = Client::new(server.uri(), "invalid", "test")
            .status()
            .await
            .unwrap_err();

        assert_eq!(error.status(), Some(reqwest::StatusCode::UNAUTHORIZED));
    }

    #[tokio::test]
    async fn returns_deserialization_errors() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/2/status"))
            .respond_with(ResponseTemplate::new(200).set_body_string("{invalid json"))
            .mount(&server)
            .await;

        let error = Client::new(server.uri(), "test-token", "test")
            .status()
            .await
            .unwrap_err();

        assert!(error.is_decode());
    }

    #[tokio::test]
    async fn searches_entities_with_highlights() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/2/entities"))
            .and(query_param("q", "time"))
            .and(query_param("limit", "30"))
            .and(query_param("highlight", "true"))
            .and(query_param("dehydrate", "true"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [{
                    "id": "doc-1",
                    "schema": "Document",
                    "caption": "Example document",
                    "collection": {"id": "1", "label": "Documents"},
                    "highlight": {"bodyText": ["An example <em>time</em> snippet"]}
                }],
                "total": 1,
                "query_q": "time"
            })))
            .mount(&server)
            .await;

        let response = Client::new(server.uri(), "test-token", "test")
            .search("time", 30)
            .await
            .unwrap();

        assert_eq!(response.total, 1);
        assert_eq!(response.results[0].caption, "Example document");
        assert_eq!(
            response.results[0].highlight["bodyText"][0],
            "An example <em>time</em> snippet"
        );
    }

    #[tokio::test]
    async fn allows_an_empty_search_query() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/2/entities"))
            .and(query_param("q", ""))
            .and(query_param("limit", "30"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [{
                    "id": "doc-1",
                    "schema": "Document",
                    "caption": "Unhighlighted document",
                    "collection": null,
                    "highlight": null
                }],
                "total": 1,
                "query_q": null
            })))
            .mount(&server)
            .await;

        let response = Client::new(server.uri(), "test-token", "test")
            .search("", 30)
            .await
            .unwrap();

        assert_eq!(response.total, 1);
        assert!(response.query_q.is_empty());
        assert!(response.results[0].highlight.is_empty());
    }

    #[tokio::test]
    async fn searches_investigations() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/2/collections"))
            .and(query_param("q", "fraud"))
            .and(query_param("limit", "30"))
            .and(query_param("filter:category", "casefile"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [{
                    "id": "42",
                    "label": "Fraud investigation",
                    "created_at": "2026-07-01T10:00:00Z",
                    "updated_at": "2026-07-30T12:00:00Z",
                    "count": 1234,
                    "creator": {"id": "7", "name": "Ada Lovelace"}
                }],
                "total": 1,
                "query_q": "fraud"
            })))
            .mount(&server)
            .await;

        let response = Client::new(server.uri(), "test-token", "test")
            .investigations("fraud", 30)
            .await
            .unwrap();

        assert_eq!(response.total, 1);
        assert_eq!(response.results[0].count, 1234);
        assert_eq!(
            response.results[0]
                .creator
                .as_ref()
                .map(|role| role.name.as_str()),
            Some("Ada Lovelace")
        );
    }
}
