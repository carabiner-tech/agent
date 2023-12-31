use std::{error::Error, fs::File, io::Write, path::PathBuf};

use crate::operations::fs::utils::ensure_relative;
use poem_openapi::Object;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct CreateFileRequest {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct CreateFileResponse {
    pub success: bool,
}

impl CreateFileRequest {
    pub async fn process(self) -> Result<CreateFileResponse, Box<dyn Error>> {
        let path = ensure_relative(PathBuf::from(self.path)).await?;
        let mut file = File::create(path)?;
        file.write_all(self.content.as_bytes())?;
        Ok(CreateFileResponse { success: true })
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use tempfile::TempDir;

    use super::*;

    #[rstest::fixture]
    fn tmp_dir() -> TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        dir
    }

    #[rstest::rstest]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_create_file_success(tmp_dir: TempDir) {
        let file_path = tmp_dir.path().join("new_file.txt");
        let request = CreateFileRequest {
            path: file_path.to_str().unwrap().to_string(),
            content: "Hello, world!".to_string(),
        };
        let response = request.process().await.unwrap();
        assert!(response.success);
        assert_eq!(read_to_string(file_path).unwrap(), "Hello, world!");
    }
}
