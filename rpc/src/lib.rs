#[macro_use]
mod macros;
use enum_as_inner::EnumAsInner;
use serde::{Deserialize, Serialize};
pub mod operations;

// re-export of request/responses
pub use operations::{
    fs::list_files::{ListFilesRequest, ListFilesResponse},
    fs::read_file::{ReadFileRequest, ReadFileResponse},
    time::{SystemTimeRequest, SystemTimeResponse},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcMessage<T> {
    pub id: uuid::Uuid,
    pub payload: T,
}

define_rpc!(
    ListFiles(ListFilesRequest, ListFilesResponse),
    ReadFile(ReadFileRequest, ReadFileResponse),
    SystemTime(SystemTimeRequest, SystemTimeResponse)
);
