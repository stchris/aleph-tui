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
        matchers::{header, method, path},
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
}
