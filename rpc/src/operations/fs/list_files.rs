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
    pub depth: i32,
    pub ignore_hidden: bool,
}

impl Default for ListFilesRequest {
    fn default() -> Self {
        Self {
            path: ".".to_string(),
            depth: 0,
            ignore_hidden: true,
        }
    }
}

struct Directory {
    path: PathBuf,
    files: Vec<String>,
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

fn process_directory_entry(
    entry: fs::DirEntry,
    dir: &mut Directory,
    queue: &mut VecDeque<Directory>,
    untraversed_dirs: &mut Vec<Directory>,
    max_depth: i32,
    ignore_hidden: bool,
) {
    let path = entry.path();

    if ignore_hidden && is_hidden(&path) {
        return;
    }

    if path.is_file() {
        if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
            dir.files.push(file_name.to_string());
        }
    } else if path.is_dir() {
        if dir.depth == max_depth {
            untraversed_dirs.push(Directory {
                path: path.clone(),
                files: Vec::new(),
                depth: dir.depth + 1,
            });
        } else {
            queue.push_back(Directory {
                path: path.clone(),
                files: Vec::new(),
                depth: dir.depth + 1,
            });
        }
    }
}

impl ListFilesRequest {
    pub async fn process(self) -> Result<ListFilesResponse, Box<dyn Error>> {
        println!("Processing request: {:?}", self);
        let mut directories: Vec<Directory> = Vec::new();
        let mut untraversed_dirs: Vec<Directory> = Vec::new();
        let mut queue: VecDeque<Directory> = VecDeque::new();
        queue.push_back(Directory {
            path: self.path.into(),
            files: Vec::new(),
            depth: self.depth,
        });
        while let Some(mut dir) = queue.pop_front() {
            if dir.depth > self.depth {
                continue;
            }

            if let Ok(entries) = fs::read_dir(&dir.path) {
                for entry in entries.filter_map(Result::ok) {
                    process_directory_entry(
                        entry,
                        &mut dir,
                        &mut queue,
                        &mut untraversed_dirs,
                        self.depth,
                        self.ignore_hidden,
                    );
                }
            }

            directories.push(dir);
        }
        let files = directories
            .iter()
            .flat_map(|dir| dir.files.clone())
            .collect::<Vec<String>>();
        let untraversed = untraversed_dirs
            .iter()
            .map(|dir| dir.path.to_str().unwrap().to_string())
            .collect::<Vec<String>>();

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
