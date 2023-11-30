#[macro_use]
mod macros;
use enum_as_inner::EnumAsInner;
use serde::{Deserialize, Serialize};
pub mod operations;

// re-export of request/responses
pub use operations::{
    commands::run_python::{RunPythonRequest, RunPythonResponse},
    commands::rustlings::{RustlingsVerifyRequest, RustlingsVerifyResponse},
    fs::{
        create_directory::{CreateDirectoryRequest, CreateDirectoryResponse},
        create_file::{CreateFileRequest, CreateFileResponse},
        delete_content::{DeleteContentRequest, DeleteContentResponse},
        diff::{DiffRequest, DiffResponse},
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
    Diff(DiffRequest, DiffResponse),
    InsertContent(InsertContentRequest, InsertContentResponse),
    ReplaceContent(ReplaceContentRequest, ReplaceContentResponse),
    DeleteContent(DeleteContentRequest, DeleteContentResponse),
    // debug / demo
    SystemTime(SystemTimeRequest, SystemTimeResponse),
    // commands
    RunPython(RunPythonRequest, RunPythonResponse),
    RustlingsVerify(RustlingsVerifyRequest, RustlingsVerifyResponse)
);
