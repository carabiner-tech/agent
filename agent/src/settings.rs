use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use url::Url;

lazy_static! {
    static ref SETTINGS: Settings = Settings::from_config();
}

pub fn get_settings() -> &'static Settings {
    &SETTINGS
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    #[serde(default = "Settings::default_rpc_server")]
    pub rpc_server: Url,
}

impl Settings {
    pub fn from_config() -> Self {
        let builder = config::Config::builder()
            .add_source(config::Environment::default())
            .build()
            .expect("Error building settings config from file and env");
        builder
            .try_deserialize()
            .expect("Error converting config into Settings struct")
    }

    pub fn default_rpc_server() -> Url {
        Url::parse("ws://localhost:3000/ws").unwrap()
    }
}
