//! List the files and subdirectories at a given path.
//! Only allows relative paths from CWD where Agent started.
use std::{cmp::Ordering, error::Error};

use poem_openapi::Object;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Object)]
#[oai(default)]
pub struct ListFilesRequest {
    pub path: String,
}

impl Default for ListFilesRequest {
    fn default() -> Self {
        Self {
            path: ".".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Object)]
pub struct File {
    pub name: String,
    pub size: u64,
}

impl PartialOrd for File {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for File {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct ListFilesResponse {
    pub files: Vec<File>,
    pub directories: Vec<String>,
}

impl ListFilesRequest {
    pub async fn process(self) -> Result<ListFilesResponse, Box<dyn Error>> {
        let mut files = vec![];
        let mut directories = vec![];
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

        // If path doesn't exist, err out
        if !path.exists() {
            return Err(format!("No such file or directory: {}", path.display()).into());
        }

        // If path is a file, err out
        if path.is_file() {
            return Err(format!("Path is a file, not a directory: {}", path.display()).into());
        }

        // Okay we're probably in the happy path finally
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            if path.is_dir() {
                directories.push(name);
            } else {
                let size = std::fs::metadata(&path)?.len();
                files.push(File { name, size });
            }
        }

        Ok(ListFilesResponse { files, directories })
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
    async fn test_req_path_has_files(_tmp_dir: TempDir) {
        let mut py_file = File::create("main.py").unwrap();
        let py_code = r#"print("hello world")"#;
        writeln!(py_file, "{}", py_code).unwrap();

        let mut rs_file = File::create("main.rs").unwrap();
        let rs_code = r#"fn main() { println!("hello world"); }"#;
        writeln!(rs_file, "{}", rs_code).unwrap();

        let req = ListFilesRequest::default();
        let mut resp = req.process().await.unwrap();
        assert_eq!(resp.files.len(), 2);
        // sort files by name before asserting order
        resp.files.sort();
        assert_eq!(resp.files[0].name, "main.py");
        assert_eq!(resp.files[0].size, 21);
        assert_eq!(resp.files[1].name, "main.rs");
        assert_eq!(resp.files[1].size, 39);
    }

    #[rstest::rstest]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_req_path_has_sub_dir(_tmp_dir: TempDir) {
        std::fs::create_dir("test_subdir").unwrap();
        let req = ListFilesRequest::default();
        let resp = req.process().await.unwrap();
        assert_eq!(resp.directories.len(), 1);
        assert_eq!(resp.directories[0], "test_subdir");
    }

    #[rstest::rstest]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_req_path_is_not_relative(_tmp_dir: TempDir) {
        let req = ListFilesRequest {
            path: "/tmp".to_string(),
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
    async fn test_req_path_is_a_file(_tmp_dir: TempDir) {
        let mut py_file = File::create("main.py").unwrap();
        let py_code = r#"print("hello world")"#;
        writeln!(py_file, "{}", py_code).unwrap();

        let req = ListFilesRequest {
            path: "main.py".to_string(),
        };
        let resp = req.process().await;
        assert!(resp.is_err());
        assert_eq!(
            resp.unwrap_err().to_string(),
            "Path is a file, not a directory: main.py"
        );
    }

    #[rstest::rstest]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_req_path_does_not_exist(_tmp_dir: TempDir) {
        let req = ListFilesRequest {
            path: "does_not_exist".to_string(),
        };
        let resp = req.process().await;
        assert!(resp.is_err());
        assert_eq!(
            resp.unwrap_err().to_string(),
            "No such file or directory: does_not_exist"
        );
    }
}
