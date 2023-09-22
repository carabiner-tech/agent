//! Insert new lines in a file
use std::{error::Error, path::PathBuf};

use poem_openapi::Object;
use serde::{Deserialize, Serialize};

use crate::operations::fs::utils::read_lines;

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
        let path = PathBuf::from(&self.path);
        // This is a little weird since utils.read_lines is expecting a start and end line, but
        // still using it to handle the redundant checks for relative path, dropping to 0-based
        // index, etc etc.
        let start_line = self.line;
        let end_line = self.line;
        let (mut lines, start_line, _end_line) =
            read_lines(path.clone(), start_line, end_line).await?;
        lines.insert(start_line, self.content);
        let content = lines.join("\n");
        tokio::fs::write(path, &content).await?;
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
    async fn test_insert_one_line(_tmp_dir: TempDir) {
        Write::write_all(
            &mut File::create("test.txt").unwrap(),
            b"line1\nline2\nline3",
        )
        .unwrap();
        let path = PathBuf::from("test.txt");
        let request = InsertContentRequest {
            path: path.to_str().unwrap().to_string(),
            content: "new line".to_string(),
            line: 2,
        };
        let response = request.process().await.unwrap();
        assert_eq!(response.content, "line1\nnew line\nline2\nline3");
    }

    #[rstest::rstest]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_insert_multiple_lines(_tmp_dir: TempDir) {
        Write::write_all(
            &mut File::create("test.txt").unwrap(),
            b"line1\nline2\nline3",
        )
        .unwrap();
        let path = PathBuf::from("test.txt");
        let request = InsertContentRequest {
            path: path.to_str().unwrap().to_string(),
            content: "new line\nanother new line".to_string(),
            line: 2,
        };
        let response = request.process().await.unwrap();
        assert_eq!(
            response.content,
            "line1\nnew line\nanother new line\nline2\nline3"
        );
    }
}
