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
mod tests {
    use super::*;

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
        assert!(cfg.default == "foo")
    }
}

#[derive(Debug, PartialEq)]
pub enum CurrentView {
    Main,
    ProfileSwitcher,
}

impl App {
    pub fn new() -> color_eyre::Result<Self> {
        let mut config_path = home::home_dir().ok_or_else(|| eyre!("Could not determine home directory"))?;
        config_path.push(".config/aleph-tui.toml");

        let config = read_to_string(&config_path)
            .map_err(|e| eyre!("Failed to read config file at {}: {}", config_path.display(), e))?;

        let config: Config = toml::from_str(&config)
            .map_err(|e| eyre!("Failed to parse config file: {}", e))?;

        let current_profile = config
            .profiles
            .iter()
            .find(|p| p.name == config.default)
            .ok_or_else(|| eyre!("Default profile '{}' not found in configuration", config.default))?;
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
