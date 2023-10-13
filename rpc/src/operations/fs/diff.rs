use diffy::{apply, Patch};
use std::{error::Error, path::PathBuf};

use crate::operations::fs::utils::ensure_relative;
use poem_openapi::Object;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct DiffRequest {
    pub path: String,
    pub diff_str: String,
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct DiffResponse {
    pub success: bool,
}

impl DiffRequest {
    pub async fn process(self) -> Result<DiffResponse, Box<dyn Error>> {
        println!("Processing diff request: {:?}", self);
        let path = ensure_relative(PathBuf::from(self.path)).await?;
        let content = std::fs::read_to_string(&path)?;
        let patch = Patch::from_str(&self.diff_str)?;
        println!("Applying patch: {:?}", patch);
        let result = apply(&content, &patch)?;
        println!("Result: {}", result);
        std::fs::write(path, result)?;
        Ok(DiffResponse { success: true })
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
        };
        let response = request.process().await.unwrap();
        assert_eq!(response.success, true);
        // Make sure it was written to disk
        let content_on_disk = std::fs::read_to_string("test.txt").unwrap();
        assert_eq!(content_on_disk, "foo\nqux\nbaz");
    }
}
