use diffy::{apply, Patch};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct EditFileRequest {
    pub path: String,
    pub patch: String,
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct EditFileResponse {
    pub new_content: String,
}

impl EditFileRequest {
    pub async fn process(self) -> Result<EditFileResponse, Box<dyn Error>> {
        let path = PathBuf::from(&self.path);
        let original_content = fs::read_to_string(&path)?;
        println!("Incoming patch: {}", &self.patch);
        let patch = Patch::from_str(&self.patch)?;
        println!("Parsed patch: {:?}", &patch);
        let patched_content = apply(&original_content, &patch)?;
        println!("Patched content: {}", &patched_content);
        fs::write(&path, &patched_content)?;

        Ok(EditFileResponse {
            new_content: patched_content,
        })
    }
}
