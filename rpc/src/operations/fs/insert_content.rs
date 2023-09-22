//! Insert new lines in a file
use std::{error::Error, path::PathBuf};

use poem_openapi::Object;
use serde::{Deserialize, Serialize};

use crate::operations::fs::utils::{ensure_relative, read_lines};

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
        let path = ensure_relative(PathBuf::from(self.path)).await?;
        let mut lines = read_lines(&path).await?;
        // Figure out where to insert the new content now
        // If line is 0, the LLM incorrectly sent a 0-indexed line number but we should just handle
        // it, the LLM wants to prepend content to top of file
        // If the line is out of index, the LLM intended to append to bottom of file
        let line = match self.line {
            0 => 0,
            line if line > lines.len() => lines.len(),
            line => line - 1,
        };

        lines.insert(line, self.content);
        let content = lines.join("\n");
        tokio::fs::write(path, &content).await?;
        Ok(InsertContentResponse { content })
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
    async fn test_insert_one_line(_tmp_dir: TempDir) {
        let mut f = File::create("test.txt").unwrap();
        f.write_all(b"line1\nline2\nline3").unwrap();
        let request = InsertContentRequest {
            path: "test.txt".to_string(),
            content: "new line".to_string(),
            line: 2,
        };
        let response = request.process().await.unwrap();
        assert_eq!(response.content, "line1\nnew line\nline2\nline3");
        // Make sure it was written to disk
        let content_on_disk = std::fs::read_to_string("test.txt").unwrap();
        assert_eq!(content_on_disk, "line1\nnew line\nline2\nline3");
    }

    #[rstest::rstest]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_insert_multiple_lines(_tmp_dir: TempDir) {
        let mut f = File::create("test.txt").unwrap();
        f.write_all(b"line1\nline2\nline3").unwrap();
        let request = InsertContentRequest {
            path: "test.txt".to_string(),
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
