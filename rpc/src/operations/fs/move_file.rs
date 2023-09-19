use std::{fs, io};

use poem_openapi::Object;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct MoveFileRequest {
    pub src_path: String,
    pub dest_path: String,
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct MoveFileResponse {
    pub success: bool,
}

impl MoveFileRequest {
    pub async fn process(self) -> io::Result<MoveFileResponse> {
        let src_path = std::path::Path::new(&self.src_path);
        let dest_path = std::path::Path::new(&self.dest_path);

        // Only support moving files in relative paths / sub-directories of CWD
        if !src_path.is_relative() && !src_path.starts_with(std::env::current_dir()?)
            || !dest_path.is_relative() && !dest_path.starts_with(std::env::current_dir()?)
        {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Both source and destination paths must be sub-directories of the current working directory",
            ));
        }

        fs::rename(&self.src_path, &self.dest_path)?;
        Ok(MoveFileResponse { success: true })
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
    async fn test_move_file_success(tmp_dir: TempDir) {
        let src_path = tmp_dir.path().join("src_file.txt");
        let dest_path = tmp_dir.path().join("dest_file.txt");

        // Create a source file
        let mut file = File::create(&src_path).unwrap();
        file.write_all(b"Hello, world!").unwrap();

        let request = MoveFileRequest {
            src_path: src_path.to_str().unwrap().to_string(),
            dest_path: dest_path.to_str().unwrap().to_string(),
        };
        let response = request.process().await.unwrap();
        assert!(response.success);
        assert!(!src_path.exists());
        assert!(dest_path.exists());
    }
}
