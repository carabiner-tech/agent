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
        delete_content::{DeleteContentRequest, DeleteContentResponse},
        insert_content::{InsertContentRequest, InsertContentResponse},
        list_files::{ListFilesRequest, ListFilesResponse},
        move_file::{MoveFileRequest, MoveFileResponse},
        read_file::{ReadFileRequest, ReadFileResponse},
        remove_file::{RemoveFileRequest, RemoveFileResponse},
        replace_content::{ReplaceContentRequest, ReplaceContentResponse},
    },
    time::{SystemTimeRequest, SystemTimeResponse},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcMessage<T> {
    pub id: uuid::Uuid,
    pub payload: T,
}

define_rpc!(
    // Directory operations
    ListFiles(ListFilesRequest, ListFilesResponse),
    CreateDirectory(CreateDirectoryRequest, CreateDirectoryResponse),
    // File CRUD
    CreateFile(CreateFileRequest, CreateFileResponse),
    ReadFile(ReadFileRequest, ReadFileResponse),
    MoveFile(MoveFileRequest, MoveFileResponse),
    RemoveFile(RemoveFileRequest, RemoveFileResponse),
    // edit file content
    InsertContent(InsertContentRequest, InsertContentResponse),
    ReplaceContent(ReplaceContentRequest, ReplaceContentResponse),
    DeleteContent(DeleteContentRequest, DeleteContentResponse),
    // debug / demo
    SystemTime(SystemTimeRequest, SystemTimeResponse),
);
