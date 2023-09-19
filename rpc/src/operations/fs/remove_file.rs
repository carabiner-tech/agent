use serde::{Deserialize, Serialize};
use std::fs;
use std::io;

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveFileRequest {
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveFileResponse {
    pub success: bool,
}

impl RemoveFileRequest {
    pub async fn process(self) -> io::Result<RemoveFileResponse> {
        let path = std::path::Path::new(&self.path);

        // Only support removing files in relative paths / sub-directories of CWD
        if !path.is_relative() && !path.starts_with(std::env::current_dir()?) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Path must be a sub-directory of the current working directory",
            ));
        }

        fs::remove_file(&self.path)?;
        Ok(RemoveFileResponse { success: true })
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Write};
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
    async fn test_remove_file_success(tmp_dir: TempDir) {
        let file_path = tmp_dir.path().join("file_to_remove.txt");

        // Create a file to remove
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"Hello, world!").unwrap();

        let request = RemoveFileRequest {
            path: file_path.to_str().unwrap().to_string(),
        };
        let response = request.process().await.unwrap();
        assert!(response.success);
        assert!(!file_path.exists());
    }
}
