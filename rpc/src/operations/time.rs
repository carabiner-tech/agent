use std::error::Error;

use poem_openapi::Object;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct SystemTimeRequest {}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemTimeResponse {
    pub time: String,
}

impl SystemTimeRequest {
    pub async fn process(self) -> Result<SystemTimeResponse, Box<dyn Error>> {
        let time = chrono::Utc::now().to_rfc3339();
        Ok(SystemTimeResponse { time })
    }
}
