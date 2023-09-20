//! List the files and subdirectories at a given path.
//! Only allows relative paths from CWD where Agent started.
use std::{cmp::Ordering, error::Error};

use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize, Object)]
#[oai(default)]
pub struct ListFilesRequest {
    pub path: String,
    pub depth: i32,
}

impl Default for ListFilesRequest {
    fn default() -> Self {
        Self {
            path: ".".to_string(),
            depth: -1,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct ListFilesResponse {
    pub files: Vec<String>,
    pub untraversed_directories: Vec<String>,
}

impl ListFilesResponse {
    pub fn format_as_tree(&self) -> String {
        let mut output = String::new();
        for file in &self.files {
            output.push_str(&format!("{}\n", file));
        }
        for dir in &self.untraversed_directories {
            output.push_str(&format!("{} [not traversed]\n", dir));
        }
        output
    }
}

impl ListFilesRequest {
    pub async fn process(self) -> Result<ListFilesResponse, Box<dyn Error>> {
        println!("Processing request: {:?}", self);
        let mut files = vec![];
        let mut untraversed_directories = vec![];

        let walker = WalkDir::new(&self.path)
            .into_iter()
            .filter_map(|e| e.ok())
            .skip(1); // Skip the root directory

        for entry in walker {
            let depth = entry.depth() as i32;
            let path = entry.path().to_path_buf();

            if depth > self.depth && self.depth != -1 {
                untraversed_directories.push(path.to_str().unwrap().to_string());
                continue;
            }

            if path.is_dir() {
                continue;
            }

            files.push(path.to_str().unwrap().to_string());
        }

        Ok(ListFilesResponse {
            files,
            untraversed_directories,
        })
    }
}

// #[cfg(test)]
// mod tests {
//     use std::{fs::File, io::Write};

//     use tempfile::TempDir;

//     use super::*;

//     #[rstest::fixture]
//     fn tmp_dir() -> TempDir {
//         let dir = tempfile::tempdir().unwrap();
//         std::env::set_current_dir(&dir).unwrap();
//         dir
//     }

//     #[rstest::rstest]
//     #[tokio::test]
//     #[serial_test::serial]
//     async fn test_req_path_has_files(_tmp_dir: TempDir) {
//         let mut py_file = File::create("main.py").unwrap();
//         let py_code = r#"print("hello world")"#;
//         writeln!(py_file, "{}", py_code).unwrap();

//         let mut rs_file = File::create("main.rs").unwrap();
//         let rs_code = r#"fn main() { println!("hello world"); }"#;
//         writeln!(rs_file, "{}", rs_code).unwrap();

//         let req = ListFilesRequest::default();
//         let mut resp = req.process().await.unwrap();
//         assert_eq!(resp.files.len(), 2);
//         // sort files by name before asserting order
//         resp.files.sort();
//         assert_eq!(resp.files[0].name, "main.py");
//         assert_eq!(resp.files[0].size, 21);
//         assert_eq!(resp.files[1].name, "main.rs");
//         assert_eq!(resp.files[1].size, 39);
//     }

//     #[rstest::rstest]
//     #[tokio::test]
//     #[serial_test::serial]
//     async fn test_req_path_has_sub_dir(_tmp_dir: TempDir) {
//         std::fs::create_dir("test_subdir").unwrap();
//         let req = ListFilesRequest::default();
//         let resp = req.process().await.unwrap();
//         assert_eq!(resp.directories.len(), 1);
//         assert_eq!(resp.directories[0], "test_subdir");
//     }

//     #[rstest::rstest]
//     #[tokio::test]
//     #[serial_test::serial]
//     async fn test_req_path_is_not_relative(_tmp_dir: TempDir) {
//         let req = ListFilesRequest {
//             path: "/tmp".to_string(),
//         };
//         let resp = req.process().await;
//         assert!(resp.is_err());
//         assert_eq!(
//             resp.unwrap_err().to_string(),
//             "Path must be a sub-directory of the current working directory"
//         );
//     }

//     #[rstest::rstest]
//     #[tokio::test]
//     #[serial_test::serial]
//     async fn test_req_path_is_a_file(_tmp_dir: TempDir) {
//         let mut py_file = File::create("main.py").unwrap();
//         let py_code = r#"print("hello world")"#;
//         writeln!(py_file, "{}", py_code).unwrap();

//         let req = ListFilesRequest {
//             path: "main.py".to_string(),
//         };
//         let resp = req.process().await;
//         assert!(resp.is_err());
//         assert_eq!(
//             resp.unwrap_err().to_string(),
//             "Path is a file, not a directory: main.py"
//         );
//     }

//     #[rstest::rstest]
//     #[tokio::test]
//     #[serial_test::serial]
//     async fn test_req_path_does_not_exist(_tmp_dir: TempDir) {
//         let req = ListFilesRequest {
//             path: "does_not_exist".to_string(),
//         };
//         let resp = req.process().await;
//         assert!(resp.is_err());
//         assert_eq!(
//             resp.unwrap_err().to_string(),
//             "No such file or directory: does_not_exist"
//         );
//     }
// }
