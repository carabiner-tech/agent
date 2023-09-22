//! Delete one or more lines in a file.
//! Using one-based start/end lines because that's the most common approach in text editors
//! and probably the LLM training set
use std::{error::Error, path::PathBuf};

use poem_openapi::Object;
use serde::{Deserialize, Serialize};

use crate::operations::fs::utils::{ensure_relative, read_lines};

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct DeleteContentRequest {
    pub path: String,
    pub start_line: usize,
    pub end_line: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct DeleteContentResponse {
    pub content: String,
}

impl DeleteContentRequest {
    pub async fn process(self) -> Result<DeleteContentResponse, Box<dyn Error>> {
        let path = ensure_relative(PathBuf::from(self.path)).await?;
        let mut lines = read_lines(&path).await?;

        // First sanity check the start line
        let start_line = match self.start_line {
            0 => 0,
            line if line - 1 > lines.len() => {
                return Err("Start line is out of index".into());
            }
            line => line - 1,
        };
        // Figure out end line
        let end_line = match self.end_line {
            Some(end_line) => match end_line {
                line if line > lines.len() => lines.len(),
                line => line - 1,
            },
            None => start_line,
        };

        lines.drain(start_line..=end_line);
        let content = lines.join("\n");
        tokio::fs::write(path, &content).await?;

        Ok(DeleteContentResponse { content })
    }
}

#[cfg(test)]
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
    async fn test_delete_one_line(_tmp_dir: TempDir) {
        let mut f = File::create("test.txt").unwrap();
        f.write_all(b"line1\nline2\nline3\n").unwrap();
        let request = DeleteContentRequest {
            path: "test.txt".to_string(),
            start_line: 2,
            end_line: None,
        };
        let response = request.process().await.unwrap();
        assert_eq!(response.content, "line1\nline3\n");
        // Make sure it was written to disk
        let content = tokio::fs::read_to_string("test.txt").await.unwrap();
        assert_eq!(content, "line1\nline3\n");
    }

    #[rstest::rstest]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_delete_two_lines(_tmp_dir: TempDir) {
        let mut f = File::create("test.txt").unwrap();
        f.write_all(b"line1\nline2\nline3\n").unwrap();
        let request = DeleteContentRequest {
            path: "test.txt".to_string(),
            start_line: 2,
            end_line: Some(3),
        };
        let response = request.process().await.unwrap();
        assert_eq!(response.content, "line1\n");
    }
}
