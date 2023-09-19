use std::{
    fs::File,
    io::{self, Write},
};

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
    pub async fn process(self) -> io::Result<CreateFileResponse> {
        let path = std::path::Path::new(&self.path);
        // Only support creating files in relative paths / sub-directories of CWD
        if !path.is_relative() && !path.starts_with(std::env::current_dir()?) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Path must be a sub-directory of the current working directory",
            ));
        }
        let mut file = File::create(&self.path)?;
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
