//! Replace content of a file between a start and end line
use std::{error::Error, path::PathBuf};

use poem_openapi::Object;
use serde::{Deserialize, Serialize};

use crate::operations::fs::utils::read_lines;

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
        let path = PathBuf::from(&self.path);
        let end_line = match self.end_line {
            Some(end_line) => end_line,
            None => self.start_line,
        };

        let (mut lines, start_line, end_line) =
            read_lines(path.clone(), self.start_line, end_line).await?;

        let new_lines: Vec<String> = self.content.lines().map(|l| l.to_string()).collect();
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
        Write::write_all(
            &mut File::create("test.txt").unwrap(),
            b"line1\nline2\nline3\n",
        )
        .unwrap();
        let request = ReplaceContentRequest {
            path: "test.txt".to_string(),
            content: "new line".to_string(),
            start_line: 2, // should replace line2
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
        Write::write_all(
            &mut File::create("test.txt").unwrap(),
            b"line1\nline2\nline3\n",
        )
        .unwrap();
        let request = ReplaceContentRequest {
            path: "test.txt".to_string(),
            content: "new line\nnew line2".to_string(),
            start_line: 2, // should replace line2
            end_line: None,
        };
        let response = request.process().await.unwrap();
        assert_eq!(response.content, "line1\nnew line\nnew line2\nline3\n");
    }

    #[rstest::rstest]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_replace_two_lines_with_one(_tmp_dir: TempDir) {
        Write::write_all(
            &mut File::create("test.txt").unwrap(),
            b"line1\nline2\nline3\n",
        )
        .unwrap();
        let request = ReplaceContentRequest {
            path: "test.txt".to_string(),
            content: "new line".to_string(),
            start_line: 2,
            end_line: Some(3),
        };
        let response = request.process().await.unwrap();
        assert_eq!(response.content, "line1\nnew line\n");
    }
}
