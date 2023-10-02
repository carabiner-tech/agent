use diffy::{apply, Patch};
use std::{error::Error, path::PathBuf};

use crate::operations::fs::utils::ensure_relative;
use poem_openapi::Object;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct DiffRequest {
    pub path: String,
    pub diff_str: String,
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct DiffResponse {
    pub success: bool,
}

impl DiffRequest {
    pub async fn process(self) -> Result<DiffResponse, Box<dyn Error>> {
        let path = ensure_relative(PathBuf::from(self.path)).await?;
        let content = std::fs::read_to_string(&path)?;
        let patch = Patch::from_str(&self.diff_str)?;
        let result = apply(&content, &patch)?;
        std::fs::write(path, result)?;
        Ok(DiffResponse { success: true })
    }
}
