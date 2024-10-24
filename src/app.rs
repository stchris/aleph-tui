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

#[derive(Debug)]
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
    pub fn new() -> Self {
        let mut config_path = home::home_dir().expect("Couldn't figure out home dir");
        config_path.push(".config/aleph-tui.toml");
        let config = read_to_string(config_path).expect("Unable to read config file");
        let config: Config = toml::from_str(&config).expect("Unable to parse config file");
        let current_profile = config
            .profiles
            .iter()
            .find(|p| p.name == config.default)
            .expect("Unable to find default profile in configuration");
        let last_fetch = Local::now();

        Self {
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
        }
    }

    pub(crate) async fn fetch(&mut self) {
        self.is_fetching = true;
        let client = reqwest::Client::new();
        let auth_header = format!("Bearer {}", self.current_profile().token);

        let url = format!(
            "{}/api/2/status",
            self.config.profiles[self.current_profile].url
        );
        let status = client
            .get(url)
            .header(AUTHORIZATION, auth_header.to_string())
            .header(
                reqwest::header::USER_AGENT,
                format!("aleph-tui/{}", self.version),
            )
            .send()
            .await
            .expect("Failed to fetch status")
            .error_for_status()
            .expect("Status request failed")
            .json()
            .await
            .expect("Failed to get status");
        self.status = status;

        let url = format!(
            "{}/api/2/metadata",
            self.config.profiles[self.current_profile].url
        );
        let metadata = client
            .get(url)
            .header(AUTHORIZATION, auth_header)
            .header(
                reqwest::header::USER_AGENT,
                format!("aleph-tui/{}", self.version),
            )
            .send()
            .await
            .unwrap()
            .error_for_status()
            .expect("Failed to fetch metadata")
            .json()
            .await
            .expect("Failed to deserialize metadata");
        self.metadata = metadata;

        self.error_message = "".to_string();
        self.is_fetching = false;
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

    pub async fn set_profile(&mut self, profile: String) -> color_eyre::Result<()> {
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

    pub(crate) async fn profile_down(&mut self) {
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

    pub(crate) async fn profile_up(&mut self) {
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

    pub(crate) async fn collection_up(&mut self) {
        let index = self.collection_tablestate.selected().unwrap_or_default();
        if index > 0 {
            self.collection_tablestate.select(Some(index - 1));
        }
    }

    pub(crate) async fn collection_down(&mut self) {
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
        Self::new()
    }
}
