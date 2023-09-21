//! Replace content of a file between a start and end line
use std::error::Error;

use poem_openapi::Object;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct ReplaceContentRequest {
    pub path: String,
    pub content: String,
    pub start_line: usize,
    pub end_line: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct ReplaceContentResponse {
    pub content: String,
}

impl ReplaceContentRequest {
    pub async fn process(self) -> Result<ReplaceContentResponse, Box<dyn Error>> {
        let path = std::path::Path::new(&self.path);
        let start_line = self.start_line;
        let end_line = match self.end_line {
            Some(end_line) => end_line,
            None => start_line + 1,
        };
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
        let mut lines: Vec<&str> = content.split('\n').collect();
        if start_line > end_line {
            return Err("Start line must be less than end line".into());
        }
        if start_line > lines.len() {
            return Err("Start line out of index".into());
        }
        if end_line > lines.len() {
            return Err("End line out of index".into());
        }
        lines.splice(start_line..end_line, self.content.split('\n'));
        let content = lines.join("\n");

        Ok(ReplaceContentResponse { content })
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
    async fn test_replace_one_line_content(_tmp_dir: TempDir) {
        let path = PathBuf::from("test.txt");
        let mut file = File::create(&path).unwrap();
        // also prove we're maintaining trailing new lines
        file.write_all(b"line1\nline2\nline3\n\n").unwrap();
        let request = ReplaceContentRequest {
            path: path.to_str().unwrap().to_string(),
            content: "new line".to_string(),
            start_line: 1,
            end_line: None,
        };
        let response = request.process().await.unwrap();
        assert_eq!(response.content, "line1\nnew line\nline3\n\n".to_string());
    }

    #[rstest::rstest]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_replace_one_line_with_two(_tmp_dir: TempDir) {
        let path = PathBuf::from("test.txt");
        let mut file = File::create(&path).unwrap();

        file.write_all(b"line1\nline2\nline3\n").unwrap();
        let request = ReplaceContentRequest {
            path: path.to_str().unwrap().to_string(),
            content: "new line\nnew line2".to_string(),
            start_line: 1,
            end_line: None,
        };
        let response = request.process().await.unwrap();
        assert_eq!(
            response.content,
            "line1\nnew line\nnew line2\nline3\n".to_string()
        );
    }

    #[rstest::rstest]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_replace_two_lines_with_one(_tmp_dir: TempDir) {
        let path = PathBuf::from("test.txt");
        let mut file = File::create(&path).unwrap();

        file.write_all(b"line1\nline2\nline3\n").unwrap();
        let request = ReplaceContentRequest {
            path: path.to_str().unwrap().to_string(),
            content: "new line".to_string(),
            start_line: 1,
            end_line: Some(3),
        };
        let response = request.process().await.unwrap();
        assert_eq!(response.content, "line1\nnew line\n".to_string());
    }

    #[rstest::rstest]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_non_relative_path_err(_tmp_dir: TempDir) {
        let request = ReplaceContentRequest {
            path: "/".to_string(),
            content: "new line".to_string(),
            start_line: 1,
            end_line: None,
        };
        let response = request.process().await;
        assert!(response.is_err());
        assert_eq!(
            response.unwrap_err().to_string(),
            "Path must be a sub-directory of the current working directory"
        );
    }
}
