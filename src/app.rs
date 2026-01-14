use crate::models::{Metadata, Status};
use chrono::{DateTime, Local};
use color_eyre::eyre::eyre;
use ratatui::widgets::TableState;
use reqwest::header::AUTHORIZATION;
use serde::{
    de::{MapAccess, Visitor},
    Deserialize,
};
use std::fs::read_to_string;
use tokio::sync::mpsc;

/// Result of a background fetch operation
pub struct FetchResult {
    pub status: Status,
    pub metadata: Metadata,
    pub error: Option<String>,
}

pub struct App {
    pub status: Status,
    pub metadata: Metadata,
    pub config: Config,
    pub current_profile: usize,
    pub should_quit: bool,
    pub version: String,
    pub error_message: String,
    pub collection_tablestate: TableState,
    pub current_view: CurrentView,
    pub profile_tablestate: TableState,
    pub last_fetch: DateTime<Local>,
    pub is_fetching: bool,
    /// Channel receiver for background fetch results
    fetch_result_rx: mpsc::Receiver<FetchResult>,
    /// Channel sender for background fetch results
    fetch_result_tx: mpsc::Sender<FetchResult>,
}

#[derive(Clone, Debug)]
pub struct Config {
    default: String,
    pub profiles: Vec<Profile>,
    pub fetch_interval: i64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default: Default::default(),
            profiles: Default::default(),
            fetch_interval: 5,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct Profile {
    pub index: usize,
    pub name: String,
    url: String,
    token: String,
}

impl<'de> Deserialize<'de> for Config {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ConfigVisitor;

        impl<'de> Visitor<'de> for ConfigVisitor {
            type Value = Config;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("Config")
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut cfg = Config::default();
                while let Some((key, value)) = visitor.next_entry::<String, toml::Value>()? {
                    match key.as_str() {
                        "default" => {
                            cfg.default =
                                value.as_str().expect("missing default profile").to_string();
                        }
                        "fetch_interval" => {
                            cfg.fetch_interval = value
                                .as_integer()
                                .expect("fetch_interval is not an integer");
                        }
                        "profiles" => {
                            let mut profiles: Vec<Profile> = Vec::new();
                            let table = value.as_table().expect("Profiles is not a table");
                            for (index, (key, value)) in table.into_iter().enumerate() {
                                let v = value.as_table().expect("Profile is not a table");
                                let profile = Profile {
                                    name: key.to_string(),
                                    index,
                                    url: v
                                        .get("url")
                                        .expect("url missing from profile")
                                        .as_str()
                                        .expect("url is not a string")
                                        .to_string(),
                                    token: v
                                        .get("token")
                                        .expect("token missing from profile")
                                        .as_str()
                                        .expect("token is not a string")
                                        .to_string(),
                                };
                                profiles.push(profile);
                            }
                            cfg.profiles = profiles;
                        }
                        _ => {}
                    }
                }
                Ok(cfg)
            }
        }

        deserializer.deserialize_map(ConfigVisitor)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    // Test helpers
    pub fn create_test_config() -> Config {
        Config {
            default: "test".to_string(),
            profiles: vec![
                Profile {
                    index: 0,
                    name: "test".to_string(),
                    url: "http://localhost:8080".to_string(),
                    token: "test-token".to_string(),
                },
                Profile {
                    index: 1,
                    name: "prod".to_string(),
                    url: "http://prod.example.com".to_string(),
                    token: "prod-token".to_string(),
                },
            ],
            fetch_interval: 5,
        }
    }

    pub fn create_test_app() -> App {
        let config = create_test_config();
        let (tx, rx) = mpsc::channel(1);

        App {
            status: Status::default(),
            metadata: Metadata::default(),
            config,
            current_profile: 0,
            should_quit: false,
            version: "0.5.0-test".to_string(),
            error_message: String::default(),
            collection_tablestate: TableState::default(),
            current_view: CurrentView::Main,
            profile_tablestate: TableState::default(),
            last_fetch: Local::now(),
            is_fetching: false,
            fetch_result_rx: rx,
            fetch_result_tx: tx,
        }
    }

    pub fn create_test_app_with_collections() -> App {
        let mut app = create_test_app();
        let test = read_to_string("testdata/results.json").unwrap();
        app.status = serde_json::from_str(&test).unwrap();
        app
    }

    // Configuration deserialization tests
    #[test]
    fn test_de_profiles() {
        let raw = r#"
        default = "foo"

        [profiles]
            [profiles.one]
            url = "url1"
            token = "token1"

            [profiles.two]
            url = "url2"
            token = "token2"
        "#;

        let cfg: Config = toml::from_str(raw).unwrap();
        assert!(cfg.default == "foo");
        assert_eq!(cfg.profiles.len(), 2);
        assert_eq!(cfg.profiles[0].name, "one");
        assert_eq!(cfg.profiles[1].name, "two");
    }

    #[test]
    fn test_config_with_multiple_profiles() {
        let toml_str = r#"
            default = "prod"

            [profiles]
                [profiles.dev]
                url = "http://localhost:8080"
                token = "dev-token"

                [profiles.prod]
                url = "https://prod.example.com"
                token = "prod-token"
        "#;

        let cfg: Config = toml::from_str(toml_str).expect("Failed to parse config");
        assert_eq!(cfg.profiles.len(), 2);
        assert_eq!(cfg.default, "prod");
        assert_eq!(cfg.profiles[0].name, "dev");
        assert_eq!(cfg.profiles[0].url, "http://localhost:8080");
        assert_eq!(cfg.profiles[1].name, "prod");
        assert_eq!(cfg.profiles[1].url, "https://prod.example.com");
    }

    #[test]
    fn test_config_custom_fetch_interval() {
        let toml_str = r#"
            default = "test"
            fetch_interval = 10

            [profiles]
                [profiles.test]
                url = "http://test"
                token = "token"
        "#;

        let cfg: Config = toml::from_str(toml_str).expect("Failed to parse config");
        assert_eq!(cfg.fetch_interval, 10);
    }

    #[test]
    fn test_config_default_fetch_interval() {
        let toml_str = r#"
            default = "test"

            [profiles]
                [profiles.test]
                url = "http://test"
                token = "token"
        "#;

        let cfg: Config = toml::from_str(toml_str).expect("Failed to parse config");
        assert_eq!(cfg.fetch_interval, 5); // Default value
    }

    // Navigation tests
    #[test]
    fn test_collection_down_increments_selection() {
        let mut app = create_test_app_with_collections();
        app.collection_tablestate.select(Some(0));
        app.collection_down();
        assert_eq!(app.collection_tablestate.selected(), Some(1));
    }

    #[test]
    fn test_collection_down_at_boundary() {
        let mut app = create_test_app_with_collections();
        let max_index = app.status.results.len();
        app.collection_tablestate.select(Some(max_index));
        app.collection_down();
        // Should not go beyond the list
        assert_eq!(app.collection_tablestate.selected(), Some(max_index));
    }

    #[test]
    fn test_collection_up_decrements_selection() {
        let mut app = create_test_app_with_collections();
        app.collection_tablestate.select(Some(2));
        app.collection_up();
        assert_eq!(app.collection_tablestate.selected(), Some(1));
    }

    #[test]
    fn test_collection_up_at_zero() {
        let mut app = create_test_app_with_collections();
        app.collection_tablestate.select(Some(0));
        app.collection_up();
        // Should not go below 0
        assert_eq!(app.collection_tablestate.selected(), Some(0));
    }

    #[test]
    fn test_profile_down_switches_profile() {
        let mut app = create_test_app();
        assert_eq!(app.current_profile, 0);
        app.profile_down();
        assert_eq!(app.current_profile, 1);
    }

    #[test]
    fn test_profile_down_clears_state() {
        let mut app = create_test_app_with_collections();
        app.error_message = "Some error".to_string();

        app.profile_down();

        // Verify status and metadata are cleared
        assert_eq!(app.status.results.len(), 0);
        assert_eq!(app.error_message, "");
    }

    #[test]
    fn test_profile_up_switches_profile() {
        let mut app = create_test_app();
        app.current_profile = 1;
        app.profile_up();
        assert_eq!(app.current_profile, 0);
    }

    #[test]
    fn test_profile_up_at_zero() {
        let mut app = create_test_app();
        assert_eq!(app.current_profile, 0);
        app.profile_up();
        // Should not go below 0
        assert_eq!(app.current_profile, 0);
    }

    #[test]
    fn test_set_profile_by_name() {
        let mut app = create_test_app();
        app.set_profile("prod".to_string())
            .expect("Failed to set profile");
        assert_eq!(app.current_profile().name, "prod");
        assert_eq!(app.current_profile, 1);
    }

    #[test]
    fn test_set_profile_nonexistent() {
        let mut app = create_test_app();
        let result = app.set_profile("nonexistent".to_string());
        assert!(result.is_err());
    }

    // Profile selector tests
    #[test]
    fn test_toggle_profile_selector() {
        let mut app = create_test_app();
        assert_eq!(app.current_view, CurrentView::Main);

        app.toggle_profile_selector();
        assert_eq!(app.current_view, CurrentView::ProfileSwitcher);

        app.toggle_profile_selector();
        assert_eq!(app.current_view, CurrentView::Main);
    }

    #[test]
    fn test_show_profile_selector() {
        let mut app = create_test_app();
        assert!(!app.show_profile_selector());

        app.toggle_profile_selector();
        assert!(app.show_profile_selector());
    }

    // Quit test
    #[test]
    fn test_quit() {
        let mut app = create_test_app();
        assert!(!app.should_quit);
        app.quit();
        assert!(app.should_quit);
    }

    // Fetch result polling tests
    #[test]
    fn test_poll_fetch_result_empty_channel() {
        let mut app = create_test_app();
        app.is_fetching = true;
        app.poll_fetch_result();
        // Should still be fetching since no result arrived
        assert!(app.is_fetching);
    }

    #[tokio::test]
    async fn test_poll_fetch_result_with_success() {
        let mut app = create_test_app();
        app.is_fetching = true;

        // Send a successful fetch result
        let result = FetchResult {
            status: Status::default(),
            metadata: Metadata::default(),
            error: None,
        };

        app.fetch_result_tx
            .send(result)
            .await
            .expect("Failed to send result");

        // Poll should receive it
        app.poll_fetch_result();
        assert!(!app.is_fetching);
        assert_eq!(app.error_message, "");
    }

    #[tokio::test]
    async fn test_poll_fetch_result_with_error() {
        let mut app = create_test_app();
        app.is_fetching = true;

        // Send an error result
        let result = FetchResult {
            status: Status::default(),
            metadata: Metadata::default(),
            error: Some("Network error".to_string()),
        };

        app.fetch_result_tx
            .send(result)
            .await
            .expect("Failed to send result");

        // Poll should receive it
        app.poll_fetch_result();
        assert!(!app.is_fetching);
        assert_eq!(app.error_message, "Network error");
    }

    #[test]
    fn test_start_fetch_when_already_fetching() {
        let mut app = create_test_app();
        app.is_fetching = true;

        app.start_fetch();

        // Should remain in fetching state without starting new fetch
        assert!(app.is_fetching);
    }

    #[test]
    fn test_current_profile() {
        let app = create_test_app();
        let profile = app.current_profile();
        assert_eq!(profile.name, "test");
        assert_eq!(profile.url, "http://localhost:8080");
    }

    // API client tests with wiremock
    use wiremock::{
        matchers::{header, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    #[tokio::test]
    async fn test_successful_fetch() {
        let mock_server = MockServer::start().await;

        // Mock the status endpoint
        Mock::given(method("GET"))
            .and(path("/api/2/status"))
            .and(header("authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [],
                "total": 0
            })))
            .mount(&mock_server)
            .await;

        // Mock the metadata endpoint
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
            .mount(&mock_server)
            .await;

        let result = App::do_fetch(
            mock_server.uri(),
            "test-token".to_string(),
            "0.5.0-test".to_string(),
        )
        .await;

        assert!(result.error.is_none());
        assert_eq!(result.status.total, 0);
        assert_eq!(result.metadata.status, "ok");
    }

    #[tokio::test]
    async fn test_fetch_with_invalid_auth() {
        let mock_server = MockServer::start().await;

        // Mock unauthorized response
        Mock::given(method("GET"))
            .and(path("/api/2/status"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&mock_server)
            .await;

        let result = App::do_fetch(
            mock_server.uri(),
            "invalid-token".to_string(),
            "0.5.0-test".to_string(),
        )
        .await;

        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("401"));
    }

    #[tokio::test]
    async fn test_fetch_with_network_error() {
        // Use an invalid URL that will cause a connection error
        let result = App::do_fetch(
            "http://invalid-host-that-does-not-exist-12345:9999".to_string(),
            "test-token".to_string(),
            "0.5.0-test".to_string(),
        )
        .await;

        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_fetch_with_malformed_json() {
        let mock_server = MockServer::start().await;

        // Mock response with invalid JSON
        Mock::given(method("GET"))
            .and(path("/api/2/status"))
            .respond_with(ResponseTemplate::new(200).set_body_string("{invalid json"))
            .mount(&mock_server)
            .await;

        let result = App::do_fetch(
            mock_server.uri(),
            "test-token".to_string(),
            "0.5.0-test".to_string(),
        )
        .await;

        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_fetch_with_metadata_error() {
        let mock_server = MockServer::start().await;

        // Status endpoint succeeds
        Mock::given(method("GET"))
            .and(path("/api/2/status"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [],
                "total": 0
            })))
            .mount(&mock_server)
            .await;

        // Metadata endpoint fails
        Mock::given(method("GET"))
            .and(path("/api/2/metadata"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let result = App::do_fetch(
            mock_server.uri(),
            "test-token".to_string(),
            "0.5.0-test".to_string(),
        )
        .await;

        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("500"));
    }

    #[tokio::test]
    async fn test_fetch_includes_user_agent() {
        let mock_server = MockServer::start().await;

        // Verify user agent header is sent
        Mock::given(method("GET"))
            .and(path("/api/2/status"))
            .and(header("user-agent", "aleph-tui/0.5.0-test"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [],
                "total": 0
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/api/2/metadata"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "ok",
                "maintenance": false,
                "app": {}
            })))
            .mount(&mock_server)
            .await;

        let result = App::do_fetch(
            mock_server.uri(),
            "test-token".to_string(),
            "0.5.0-test".to_string(),
        )
        .await;

        assert!(result.error.is_none());
    }

    #[tokio::test]
    async fn test_maybe_start_fetch_when_interval_elapsed() {
        let mut app = create_test_app();
        app.config.fetch_interval = 1; // 1 second
        app.last_fetch = Local::now() - chrono::Duration::seconds(2);
        app.is_fetching = false;

        app.maybe_start_fetch();

        // Should have started a fetch
        assert!(app.is_fetching);
    }

    #[tokio::test]
    async fn test_maybe_start_fetch_when_interval_not_elapsed() {
        let mut app = create_test_app();
        app.config.fetch_interval = 10; // 10 seconds
        app.last_fetch = Local::now(); // Just now
        app.is_fetching = false;

        app.maybe_start_fetch();

        // Should not have started a fetch
        assert!(!app.is_fetching);
    }

    #[tokio::test]
    async fn test_maybe_start_fetch_when_already_fetching() {
        let mut app = create_test_app();
        app.config.fetch_interval = 1;
        app.last_fetch = Local::now() - chrono::Duration::seconds(2);
        app.is_fetching = true;

        app.maybe_start_fetch();

        // Should still be fetching (no new fetch started)
        assert!(app.is_fetching);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum CurrentView {
    Main,
    ProfileSwitcher,
}

impl App {
    pub fn new() -> color_eyre::Result<Self> {
        let mut config_path =
            home::home_dir().ok_or_else(|| eyre!("Could not determine home directory"))?;
        config_path.push(".config/aleph-tui.toml");

        let config = read_to_string(&config_path).map_err(|e| {
            eyre!(
                "Failed to read config file at {}: {}",
                config_path.display(),
                e
            )
        })?;

        let config: Config =
            toml::from_str(&config).map_err(|e| eyre!("Failed to parse config file: {}", e))?;

        let current_profile = config
            .profiles
            .iter()
            .find(|p| p.name == config.default)
            .ok_or_else(|| {
                eyre!(
                    "Default profile '{}' not found in configuration",
                    config.default
                )
            })?;
        let last_fetch = Local::now();

        // Create channel for background fetch results (buffer size of 1 since we only have one fetch at a time)
        let (fetch_result_tx, fetch_result_rx) = mpsc::channel(1);

        Ok(Self {
            status: Status::default(),
            config: config.clone(),
            current_profile: current_profile.index,
            should_quit: false,
            version: env!("CARGO_PKG_VERSION").to_string(),
            error_message: String::default(),
            collection_tablestate: TableState::default(),
            current_view: CurrentView::Main,
            profile_tablestate: TableState::default(),
            last_fetch,
            metadata: Metadata::default(),
            is_fetching: false,
            fetch_result_rx,
            fetch_result_tx,
        })
    }

    /// Start a background fetch operation. This is non-blocking and returns immediately.
    /// Call `poll_fetch_result()` to check for and apply results.
    pub fn start_fetch(&mut self) {
        if self.is_fetching {
            return; // Already fetching, don't start another
        }
        self.is_fetching = true;

        let tx = self.fetch_result_tx.clone();
        let url = self.config.profiles[self.current_profile].url.clone();
        let token = self.current_profile().token.clone();
        let version = self.version.clone();

        tokio::spawn(async move {
            let result = Self::do_fetch(url, token, version).await;
            // Ignore send error - receiver may have been dropped if app is shutting down
            let _ = tx.send(result).await;
        });
    }

    /// Perform the actual fetch operation. This runs in a background task.
    async fn do_fetch(base_url: String, token: String, version: String) -> FetchResult {
        let client = reqwest::Client::new();
        let auth_header = format!("Bearer {}", token);
        let user_agent = format!("aleph-tui/{}", version);

        // Fetch status
        let status_url = format!("{}/api/2/status", base_url);
        let status_result: Result<Status, reqwest::Error> = async {
            client
                .get(&status_url)
                .header(AUTHORIZATION, &auth_header)
                .header(reqwest::header::USER_AGENT, &user_agent)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await
        }
        .await;

        // Fetch metadata
        let metadata_url = format!("{}/api/2/metadata", base_url);
        let metadata_result: Result<Metadata, reqwest::Error> = async {
            client
                .get(&metadata_url)
                .header(AUTHORIZATION, &auth_header)
                .header(reqwest::header::USER_AGENT, &user_agent)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await
        }
        .await;

        match (status_result, metadata_result) {
            (Ok(status), Ok(metadata)) => FetchResult {
                status,
                metadata,
                error: None,
            },
            (Err(e), _) => FetchResult {
                status: Status::default(),
                metadata: Metadata::default(),
                error: Some(e.to_string()),
            },
            (_, Err(e)) => FetchResult {
                status: Status::default(),
                metadata: Metadata::default(),
                error: Some(e.to_string()),
            },
        }
    }

    /// Poll for completed fetch results. This is non-blocking.
    /// If a fetch has completed, applies the results to the app state.
    pub fn poll_fetch_result(&mut self) {
        match self.fetch_result_rx.try_recv() {
            Ok(result) => {
                self.status = result.status;
                self.metadata = result.metadata;
                self.error_message = result.error.unwrap_or_default();
                self.is_fetching = false;
                self.last_fetch = Local::now();
            }
            Err(mpsc::error::TryRecvError::Empty) => {
                // No result yet, that's fine
            }
            Err(mpsc::error::TryRecvError::Disconnected) => {
                // Channel closed, shouldn't happen in normal operation
                self.is_fetching = false;
            }
        }
    }

    /// Check if enough time has elapsed since last fetch and start a new fetch if needed.
    /// This is non-blocking.
    pub fn maybe_start_fetch(&mut self) {
        let elapsed = Local::now() - self.last_fetch;
        if elapsed.num_seconds() > self.config.fetch_interval && !self.is_fetching {
            self.start_fetch();
        }
    }

    pub fn current_profile(&self) -> Profile {
        self.config.profiles[self.current_profile].clone()
    }

    pub fn toggle_profile_selector(&mut self) {
        self.current_view = match self.current_view {
            CurrentView::Main => CurrentView::ProfileSwitcher,
            CurrentView::ProfileSwitcher => CurrentView::Main,
        }
    }

    pub fn show_profile_selector(&self) -> bool {
        self.current_view == CurrentView::ProfileSwitcher
    }

    pub fn set_profile(&mut self, profile: String) -> color_eyre::Result<()> {
        let p = self.config.profiles.iter().find(|p| p.name == profile);
        match p {
            Some(p) => {
                self.profile_tablestate.select(Some(p.index));
                self.current_profile = p.index;
                Ok(())
            }
            None => Err(eyre!("Profile '{:?}' not found", profile)),
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub(crate) fn profile_down(&mut self) {
        if self.current_profile().index < self.config.profiles.len()
            && self
                .config
                .profiles
                .get(self.current_profile().index + 1)
                .is_some()
        {
            self.current_profile += 1;
            self.clear_state();
        }
    }

    pub(crate) fn profile_up(&mut self) {
        if self.current_profile().index > 0
            && self
                .config
                .profiles
                .get(self.current_profile().index - 1)
                .is_some()
        {
            self.current_profile -= 1;
            self.clear_state();
        }
    }

    pub(crate) fn collection_up(&mut self) {
        let index = self.collection_tablestate.selected().unwrap_or_default();
        if index > 0 {
            self.collection_tablestate.select(Some(index - 1));
        }
    }

    pub(crate) fn collection_down(&mut self) {
        let index = self.collection_tablestate.selected().unwrap_or_default();
        if index < self.status.results.len() {
            self.collection_tablestate.select(Some(index + 1));
        }
    }

    fn clear_state(&mut self) {
        self.status = Status::default();
        self.metadata = Metadata::default();
        self.error_message = String::default();
    }

    pub(crate) fn print_version(&self) {
        println!("aleph-tui {}", self.version);
    }

    pub(crate) fn print_help(&self) {
        println!("aleph-tui");
        println!();
        println!("USAGE");
        println!("aleph-tui [PROFILE]");
        println!();
        println!("OPTIONS");
        println!("--version   Print version");
        println!("--help      Show help");
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new().expect("Failed to create default App")
    }
}
