# TODO: create pyo3 bindings for rpc lib instead of having to redefine models here
import uuid
from typing import Annotated, List, Literal, Union

from pydantic import BaseModel, Field, field_serializer, field_validator


class SystemTimeRequest(BaseModel):
    type: Literal["SystemTime"]


class SystemTimeResponse(BaseModel):
    type: Literal["SystemTime"]
    time: str


class ListFilesRequest(BaseModel):
    type: Literal["ListFiles"]
    path: str = "."
    max_depth: int = 1


class ListFilesResponse(BaseModel):
    type: Literal["ListFiles"]
    files: List[str]
    untraversed: List[str]


class ReadFileRequest(BaseModel):
    type: Literal["ReadFile"]
    path: str


class ReadFileResponse(BaseModel):
    type: Literal["ReadFile"]
    content: str


class RpcError(BaseModel):
    type: Literal["RpcError"]
    e: str


RpcRequest = Union[SystemTimeRequest, ListFilesRequest, ReadFileRequest]
RpcResponse = Annotated[
    Union[SystemTimeResponse, ListFilesResponse, ReadFileResponse, RpcError],
    Field(discriminator="type"),
]


class Message(BaseModel):
    id: uuid.UUID
    payload: Union[RpcRequest, RpcResponse]
