//! Return file contents as a string
use std::error::Error;

use poem_openapi::Object;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Object)]
#[oai(default)]
pub struct ReadFileRequest {
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct ReadFileResponse {
    pub content: String,
}

impl ReadFileRequest {
    pub async fn process(self) -> Result<ReadFileResponse, Box<dyn Error>> {
        let path = std::path::Path::new(&self.path);
        // Only support checking relative paths / sub-directories of CWD
        match path.is_relative() {
            true => {}
            false => {
                if !path.starts_with(std::env::current_dir()?) {
                    return Err(
                        "Path must be a sub-directory of the current working directory".into(),
                    );
                }
            }
        }
        let content = tokio::fs::read_to_string(path).await?;
        Ok(ReadFileResponse { content })
    }
}
