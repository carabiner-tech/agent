//! List the files and subdirectories at a given path.
//! Only allows relative paths from CWD where Agent started.
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Object)]
#[oai(default)]
pub struct ListFilesRequest {
    pub path: String,
    pub max_depth: i32,
    pub ignore_hidden: bool,
}

impl Default for ListFilesRequest {
    fn default() -> Self {
        Self {
            path: ".".to_string(),
            max_depth: 3,
            ignore_hidden: true,
        }
    }
}

#[derive(Debug)]
struct Directory {
    path: PathBuf,
    files: Vec<PathBuf>,
    depth: i32,
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct ListFilesResponse {
    pub files: Vec<String>,
    pub untraversed: Vec<String>,
}

impl ListFilesResponse {
    pub fn format_as_tree(&self) -> String {
        "TODO".to_string()
    }
}

fn is_hidden(path: &PathBuf) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

impl ListFilesRequest {
    pub async fn process(self) -> Result<ListFilesResponse, Box<dyn Error>> {
        println!("Processing request: {:?}", self);
        let mut directories: Vec<Directory> = Vec::new();
        let mut untraversed_dirs: Vec<PathBuf> = Vec::new();
        let mut queue: VecDeque<Directory> = VecDeque::new();
        queue.push_back(Directory {
            path: self.path.into(),
            files: Vec::new(),
            depth: 0,
        });
        while let Some(mut dir) = queue.pop_front() {
            let entries = match fs::read_dir(&dir.path) {
                Ok(entries) => entries,
                Err(_) => continue,
            };
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();

                if self.ignore_hidden && is_hidden(&path) {
                    continue;
                }
                if path.is_file() {
                    dir.files.push(path);
                } else if path.is_dir() {
                    if dir.depth < self.max_depth {
                        let new_dir = Directory {
                            path,
                            files: Vec::new(),
                            depth: dir.depth + 1,
                        };
                        queue.push_back(new_dir);
                    } else {
                        untraversed_dirs.push(path);
                    }
                }
            }
            directories.push(dir);
        }
        let mut files: Vec<String> = directories
            .iter()
            .flat_map(|dir| {
                dir.files.iter().map(|path| {
                    path.strip_prefix(&dir.path)
                        .unwrap()
                        .to_string_lossy()
                        .to_string()
                })
            })
            .collect();
        files.sort();
        let untraversed: Vec<String> = untraversed_dirs
            .iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect();
        Ok(ListFilesResponse { files, untraversed })
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
