//! Insert new lines in a file
use std::error::Error;

use poem_openapi::Object;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct InsertContentRequest {
    pub path: String,
    pub content: String,
    pub line: usize,
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct InsertContentResponse {
    pub content: String,
}

impl InsertContentRequest {
    pub async fn process(self) -> Result<InsertContentResponse, Box<dyn Error>> {
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
        let mut lines: Vec<&str> = content.lines().collect();
        let mut line_no = self.line;
        if line_no > lines.len() {
            line_no = lines.len();
        }
        lines.insert(line_no, &self.content);
        let content = lines.join("\n");
        Ok(InsertContentResponse { content })
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Write, path::PathBuf};

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
    async fn test_insert_content(_tmp_dir: TempDir) {
        let path = PathBuf::from("test.txt");
        let mut file = File::create(&path).unwrap();
        file.write_all(b"line1\nline2\nline3").unwrap();
        let request = InsertContentRequest {
            path: path.to_str().unwrap().to_string(),
            content: "new line".to_string(),
            line: 1,
        };
        let response = request.process().await.unwrap();
        assert_eq!(
            response.content,
            "line1\nnew line\nline2\nline3".to_string()
        );
    }

    #[rstest::rstest]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_err_on_non_relative_director(_tmp_dir: TempDir) {
        let req = InsertContentRequest {
            path: "/".to_string(),
            content: "new line".to_string(),
            line: 1,
        };
        let resp = req.process().await;
        assert!(resp.is_err());
        assert_eq!(
            resp.unwrap_err().to_string(),
            "Path must be a sub-directory of the current working directory"
        );
    }

    #[rstest::rstest]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_line_out_of_index(_tmp_dir: TempDir) {
        let path = PathBuf::from("test.txt");
        let mut file = File::create(&path).unwrap();
        file.write_all(b"line1\nline2\nline3").unwrap();
        let request = InsertContentRequest {
            path: path.to_str().unwrap().to_string(),
            content: "new line".to_string(),
            line: 4,
        };
        let response = request.process().await.unwrap();
        // we should have made the out of index line just append to bottom of file
        assert_eq!(
            response.content,
            "line1\nline2\nline3\nnew line".to_string()
        );
    }
}
