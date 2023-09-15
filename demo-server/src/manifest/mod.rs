use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::settings::get_settings;

lazy_static! {
    static ref MANIFEST: Manifest = Manifest::from_config();
}

pub fn get_manifest() -> &'static Manifest {
    &MANIFEST
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Manifest {
    schema_version: String,
    name_for_human: String,
    name_for_model: String,
    logo_url: Url,
    contact_email: String,
    legal_info_url: String,
    api: Api,
    auth: Auth,
    description_for_human: String,
    description_for_model: String,
}

impl Manifest {
    pub fn from_config() -> Self {
        let settings = get_settings();

        let builder = config::Config::builder()
            .set_default("logo_url", settings.logo_url().to_string())
            .unwrap()
            .set_default("api.url", settings.openapi_json_url().to_string())
            .unwrap()
            .add_source(config::File::with_name("manifest.yml"))
            .add_source(config::Environment::with_prefix("MANIFEST").separator("_"))
            .build()
            .expect("Error building manifest config from file and env");
        let manifest: Self = builder
            .try_deserialize()
            .expect("Error converting config into Manifest struct");
        manifest
            .auth
            .clone() // TODO: Is there a better way to do this, without cloning?
            .validate()
            .expect("Error validating auth section");
        manifest
    }
}

// url should point to your openapi.json location
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Api {
    #[serde(rename = "type")]
    _type: String,
    url: Url,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Auth {
    #[serde(rename = "type")]
    auth_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_url: Option<Url>, // Redirect on first step OAuth
    #[serde(skip_serializing_if = "Option::is_none")]
    authorization_url: Option<Url>, // POST on second step of OAuth
    #[serde(skip_serializing_if = "Option::is_none")]
    authorization_content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    verification_tokens: Option<VerificationTokens>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct VerificationTokens {
    openai: String,
}

impl Auth {
    fn validate(self) -> Result<(), String> {
        if self.auth_type == "oauth" && (self.client_url.is_none()
                || self.authorization_url.is_none()
                || self.authorization_content_type.is_none()
                || self.scope.is_none() || self.verification_tokens.is_none()) {
            return Err("Missing required fields for OAuth".to_string());
        }
        Ok(())
    }
}
