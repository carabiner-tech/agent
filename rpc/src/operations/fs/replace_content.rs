//! Replace content of a file between a start and end line
use std::{error::Error, path::PathBuf};

use poem_openapi::Object;
use serde::{Deserialize, Serialize};

use crate::operations::fs::utils::{ensure_relative, read_lines};

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct ReplaceContentRequest {
    pub path: String,
    pub content: String,
    pub start_line: usize,
    pub end_line: Option<usize>, // Empty to replace just a single line
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct ReplaceContentResponse {
    pub content: String,
}

impl ReplaceContentRequest {
    pub async fn process(self) -> Result<ReplaceContentResponse, Box<dyn Error>> {
        let path = ensure_relative(PathBuf::from(self.path)).await?;
        let mut lines = read_lines(&path).await?;

        // First sanity check the start line
        // - help the LLM if it sent 0, clearly trying to change the top line in the file
        // - raise error if start line is out of index after downcasting to 0-based index
        let start_line = match self.start_line {
            0 => 0,
            line => line - 1,
        };
        if start_line >= lines.len() {
            return Err("Start line is out of index".into());
        }

        // Figure out end line
        // - if it wasn't passed in, make it same as start line to replace a single line
        // - if it out of index, make it the last line in the file
        let end_line = match self.end_line {
            Some(end_line) => match end_line {
                line if line >= lines.len() => lines.len() - 1,
                line => line - 1,
            },
            None => start_line,
        };

        let new_lines: Vec<String> = self.content.split("\n").map(|l| l.to_string()).collect();

        // splice in new lines, use inclusive range (=end_line)
        lines.splice(start_line..=end_line, new_lines);
        let content = lines.join("\n");
        tokio::fs::write(path, &content).await?;

        Ok(ReplaceContentResponse { content })
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
    async fn test_replace_one_line_content(_tmp_dir: TempDir) {
        let mut f = File::create("test.txt").unwrap();
        f.write_all(b"line1\nline2\nline3\n").unwrap();
        let request = ReplaceContentRequest {
            path: "test.txt".to_string(),
            content: "new line".to_string(),
            start_line: 2,
            end_line: None,
        };
        let response = request.process().await.unwrap();
        assert_eq!(response.content, "line1\nnew line\nline3\n");
        // Make sure it was written to disk
        let content = tokio::fs::read_to_string("test.txt").await.unwrap();
        assert_eq!(content, "line1\nnew line\nline3\n");
    }

    #[rstest::rstest]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_replace_one_line_with_two(_tmp_dir: TempDir) {
        let mut f = File::create("test.txt").unwrap();
        f.write_all(b"line1\nline2\nline3\n").unwrap();
        let request = ReplaceContentRequest {
            path: "test.txt".to_string(),
            content: "new line\nnew line2".to_string(),
            start_line: 2,
            end_line: None,
        };
        let response = request.process().await.unwrap();
        assert_eq!(response.content, "line1\nnew line\nnew line2\nline3\n");
    }

    #[rstest::rstest]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_replace_two_lines_with_one(_tmp_dir: TempDir) {
        let mut f = File::create("test.txt").unwrap();
        f.write_all(b"line1\nline2\nline3\n").unwrap();
        let request = ReplaceContentRequest {
            path: "test.txt".to_string(),
            content: "new line".to_string(),
            start_line: 2,
            end_line: Some(3),
        };
        let response = request.process().await.unwrap();
        assert_eq!(response.content, "line1\nnew line\n");
    }

    #[rstest::rstest]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_start_is_zero(_tmp_dir: TempDir) {
        let mut f = File::create("test.txt").unwrap();
        f.write_all(b"line1\nline2\nline3\n").unwrap();
        let request = ReplaceContentRequest {
            path: "test.txt".to_string(),
            content: "new line".to_string(),
            start_line: 0,
            end_line: None,
        };
        let response = request.process().await.unwrap();
        assert_eq!(response.content, "new line\nline2\nline3\n");
    }

    #[rstest::rstest]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_start_is_last_index(_tmp_dir: TempDir) {
        let mut f = File::create("test.txt").unwrap();
        f.write_all(b"line1\nline2\nline3").unwrap();
        let request = ReplaceContentRequest {
            path: "test.txt".to_string(),
            content: "new line".to_string(),
            start_line: 3,
            end_line: None,
        };
        let response = request.process().await.unwrap();
        assert_eq!(response.content, "line1\nline2\nnew line");
    }

    #[rstest::rstest]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_start_is_out_of_index(_tmp_dir: TempDir) {
        let mut f = File::create("test.txt").unwrap();
        f.write_all(b"line1\nline2\nline3").unwrap();
        let request = ReplaceContentRequest {
            path: "test.txt".to_string(),
            content: "new line".to_string(),
            start_line: 4,
            end_line: None,
        };
        let response = request.process().await;
        assert!(response.is_err());
        assert_eq!(
            response.unwrap_err().to_string(),
            "Start line is out of index"
        );
    }

    #[rstest::rstest]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_end_is_out_of_index(_tmp_dir: TempDir) {
        let mut f = File::create("test.txt").unwrap();
        f.write_all(b"line1\nline2\nline3\n").unwrap();
        let request = ReplaceContentRequest {
            path: "test.txt".to_string(),
            content: "new line".to_string(),
            start_line: 3,
            end_line: Some(10),
        };
        let response = request.process().await.unwrap();
        assert_eq!(response.content, "line1\nline2\nnew line");
    }
}
