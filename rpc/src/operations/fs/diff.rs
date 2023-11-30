use crate::operations::fs::utils::ensure_relative;
use llm_diff::FileDiff;
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufRead;
use std::io::Write;
use std::{error::Error, io::BufReader, path::PathBuf};

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct DiffRequest {
    pub commit_msg: String,
    pub path: String,
    pub diff_str: String,
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct DiffResponse {
    pub new_content: String,
}

impl DiffRequest {
    pub async fn process(self) -> Result<DiffResponse, Box<dyn Error>> {
        println!("Processing diff request: {:?}", self);
        let path = ensure_relative(PathBuf::from(self.path)).await?;
        let reader = BufReader::new(File::open(&path)?);
        let lines: Vec<String> = reader
            .lines()
            .map(|l| l.expect("could not parse line"))
            .collect();

        let diff = FileDiff::parse(&self.diff_str)?;

        let applied = diff.apply(&lines)?;
        // Write the result to disk
        let mut writer = File::create(&path)?;
        write!(writer, "{}", applied.join("\n"))?;

        let new_content = std::fs::read_to_string(&path)?;

        Ok(DiffResponse { new_content })
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
    async fn test_patch(_tmp_dir: TempDir) {
        let mut f = File::create("test.txt").unwrap();
        f.write_all(b"foo\nbar\nbaz").unwrap();
        let request = DiffRequest {
            path: "test.txt".to_string(),
            diff_str: "@@ -1,1 +1,5 @@\n foo\n-bar\n+qux\n baz".to_string(),
            commit_msg: "test".to_string(),
        };

        let expected = "foo\nqux\nbaz";
        let response = request.process().await.unwrap();
        assert_eq!(response.new_content, expected);

        // Make sure it was written to disk
        let content_on_disk = std::fs::read_to_string("test.txt").unwrap();
        assert_eq!(content_on_disk, expected);
    }
}
