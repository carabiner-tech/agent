use std::{fs, io};

use poem_openapi::Object;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct CreateDirectoryRequest {
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct CreateDirectoryResponse {
    pub success: bool,
}

impl CreateDirectoryRequest {
    pub async fn process(self) -> io::Result<CreateDirectoryResponse> {
        let path = std::path::Path::new(&self.path);
        // Only support creating directories in relative paths / sub-directories of CWD
        match path.is_relative() {
            true => {}
            false => {
                if !path.starts_with(std::env::current_dir()?) {
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "Path must be a sub-directory of the current working directory",
                    ));
                }
            }
        }
        fs::create_dir_all(&self.path)?;
        Ok(CreateDirectoryResponse { success: true })
    }
}

#[cfg(test)]
mod tests {

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
    async fn test_create_directory_success(tmp_dir: TempDir) {
        let dir_path = tmp_dir.path().join("new_directory");
        let request = CreateDirectoryRequest {
            path: dir_path.to_str().unwrap().to_string(),
        };
        let response = request.process().await.unwrap();
        assert!(response.success);
        assert!(dir_path.exists());
    }

    #[rstest::rstest]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_create_directory_not_in_cwd_or_subdirectory(_tmp_dir: TempDir) {
        let request = CreateDirectoryRequest {
            path: "/tmp/some_random_directory".to_string(),
        };
        let response = request.process().await;
        assert!(response.is_err());
        let error_message = response.unwrap_err().to_string();
        assert_eq!(
            error_message,
            "Path must be a sub-directory of the current working directory"
        );
    }

    #[rstest::rstest]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_create_nested_directory_without_parent(tmp_dir: TempDir) {
        let dir_path = tmp_dir.path().join("parent_directory/nested_directory");
        let request = CreateDirectoryRequest {
            path: dir_path.to_str().unwrap().to_string(),
        };
        let response = request.process().await.unwrap();
        assert!(response.success);
        assert!(dir_path.exists());
    }
}
