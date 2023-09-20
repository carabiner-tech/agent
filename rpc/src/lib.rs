#[macro_use]
mod macros;
use enum_as_inner::EnumAsInner;
use serde::{Deserialize, Serialize};
pub mod operations;

// re-export of request/responses
pub use operations::{
    fs::{
        create_directory::{CreateDirectoryRequest, CreateDirectoryResponse},
        create_file::{CreateFileRequest, CreateFileResponse},
        edit_file::{EditFileRequest, EditFileResponse},
        list_files::{ListFilesRequest, ListFilesResponse},
        move_file::{MoveFileRequest, MoveFileResponse},
        read_file::{ReadFileRequest, ReadFileResponse},
        remove_file::{RemoveFileRequest, RemoveFileResponse},
    },
    time::{SystemTimeRequest, SystemTimeResponse},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcMessage<T> {
    pub id: uuid::Uuid,
    pub payload: T,
}

define_rpc!(
    CreateFile(CreateFileRequest, CreateFileResponse),
    ReadFile(ReadFileRequest, ReadFileResponse),
    EditFile(EditFileRequest, EditFileResponse),
    MoveFile(MoveFileRequest, MoveFileResponse),
    RemoveFile(RemoveFileRequest, RemoveFileResponse),
    ListFiles(ListFilesRequest, ListFilesResponse),
    CreateDirectory(CreateDirectoryRequest, CreateDirectoryResponse),
    SystemTime(SystemTimeRequest, SystemTimeResponse)
);
