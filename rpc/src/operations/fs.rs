use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{Op, Operation, RpcMessage};

#[derive(Serialize, Deserialize, Debug)]
pub struct ListFilesRequest {
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct File {
    pub name: String,
    pub size: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListFilesResponse {
    pub files: Vec<File>,
    pub directories: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListFiles {
    request: Option<ListFilesRequest>,
    pub response: Option<ListFilesResponse>,
}

impl ListFiles {
    #[allow(dead_code)]
    pub fn make_request(path: String) -> RpcMessage {
        let req = ListFilesRequest { path };
        let op = Op::ListFiles(Self {
            request: Some(req),
            response: None,
        });
        RpcMessage {
            id: uuid::Uuid::new_v4(),
            op,
        }
    }
}

impl Operation for ListFiles {
    fn process(&self) -> Self {
        let path = Path::new(&self.request.as_ref().unwrap().path);
        let mut files = Vec::new();
        let mut directories = Vec::new();

        for entry in path.read_dir().unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            if path.is_dir() {
                directories.push(name);
            } else {
                let size = path.metadata().unwrap().len();
                files.push(File { name, size });
            }
        }

        let response = ListFilesResponse { files, directories };
        Self {
            request: None,
            response: Some(response),
        }
    }
}
