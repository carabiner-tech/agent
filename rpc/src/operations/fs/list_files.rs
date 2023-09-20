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
        let path = PathBuf::from(&self.path);
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
        let mut directories: Vec<Directory> = Vec::new();
        let mut untraversed_dirs: Vec<PathBuf> = Vec::new();
        let mut queue: VecDeque<Directory> = VecDeque::new();
        queue.push_back(Directory {
            path: path,
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
        // collect all files with their full relative path
        let mut files: Vec<String> = directories
            .iter()
            .flat_map(|dir| {
                dir.files
                    .iter()
                    .map(|path| path.to_string_lossy().to_string())
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

#[cfg(test)]
mod tests {
    use std::{fs, fs::File, io::Write};

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
    async fn test_nested_search(_tmp_dir: TempDir) {
        // Create nested directories and files
        fs::create_dir("level1").unwrap();
        fs::create_dir("level1/level2").unwrap();
        let mut file1 = File::create("level1/file1.txt").unwrap();
        let mut file2 = File::create("level1/level2/file2.txt").unwrap();
        writeln!(file1, "This is file1").unwrap();
        writeln!(file2, "This is file2").unwrap();

        // Create a ListFilesRequest with max_depth set to 2
        let req = ListFilesRequest {
            path: ".".to_string(),
            max_depth: 1,
            ignore_hidden: true,
        };

        // Process the request
        let resp = req.process().await.unwrap();

        // Validate the response
        assert_eq!(resp.files.len(), 1); // Should only find file1.txt
        assert_eq!(resp.untraversed.len(), 1); // level2 should be untraversed
        assert_eq!(resp.files[0], "./level1/file1.txt");
        assert_eq!(resp.untraversed[0], "./level1/level2");
    }

    #[rstest::rstest]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_err_on_non_relative_director(_tmp_dir: TempDir) {
        let req = ListFilesRequest {
            path: "/".to_string(),
            max_depth: 1,
            ignore_hidden: true,
        };
        let resp = req.process().await;
        assert!(resp.is_err());
        assert_eq!(
            resp.unwrap_err().to_string(),
            "Path must be a sub-directory of the current working directory"
        );
    }
}
