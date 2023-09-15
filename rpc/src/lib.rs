#[macro_use]
mod macros;
use enum_as_inner::EnumAsInner;
use serde::{Deserialize, Serialize};
pub mod operations;

use operations::{
    list_files::{ListFilesRequest, ListFilesResponse},
    time::{SystemTimeRequest, SystemTimeResponse},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcMessage<T> {
    pub id: uuid::Uuid,
    pub payload: T,
}

define_rpc!(
    ListFiles(ListFilesRequest, ListFilesResponse),
    SystemTime(SystemTimeRequest, SystemTimeResponse)
);
