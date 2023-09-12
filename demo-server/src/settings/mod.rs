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
    #[serde(default = "Settings::default_host")]
    pub host: String,
    #[serde(default = "Settings::default_public_url")]
    pub public_url: Url,
}

impl Settings {
    pub fn from_config() -> Self {
        let builder = config::Config::builder()
            .add_source(config::Environment::with_prefix("APP").separator("_"))
            .build()
            .expect("Error building settings config from file and env");
        builder
            .try_deserialize()
            .expect("Error converting config into Settings struct")
    }

    pub fn default_host() -> String {
        "127.0.0.1:3000".to_string()
    }

    pub fn default_public_url() -> Url {
        Url::parse("http://localhost:3000").unwrap()
    }

    pub fn logo_url(&self) -> Url {
        self.public_url.join("/logo.png").unwrap()
    }

    pub fn openapi_json_url(&self) -> Url {
        self.public_url.join("/openapi.json").unwrap()
    }
}
