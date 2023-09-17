# TODO: create pyo3 bindings for rpc lib instead of having to redefine models here
import uuid
from typing import List, Union

from pydantic import BaseModel, field_serializer, field_validator


class SystemTimeRequest(BaseModel):
    pass


class SystemTimeResponse(BaseModel):
    time: str


class ListFilesRequest(BaseModel):
    path: str = "."


class File(BaseModel):
    name: str
    size: int


class ListFilesResponse(BaseModel):
    files: List[File]
    directories: List[str]


class ReadFileRequest(BaseModel):
    path: str


class ReadFileResponse(BaseModel):
    content: str


RpcRequest = Union[SystemTimeRequest, ListFilesRequest, ReadFileRequest]
RpcResponse = Union[SystemTimeResponse, ListFilesResponse, ReadFileResponse]


class Message(BaseModel):
    id: uuid.UUID
    payload: Union[RpcRequest, RpcResponse]

    # Need these extra serializer / validator methods in order to match the structure that serde
    # creates when our RpcRequest / RpcResponse payloads are enum variants
    @field_serializer("payload")
    def serialize_payload(self, payload: RpcRequest):
        if isinstance(payload, SystemTimeRequest):
            return {"SystemTime": payload}
        elif isinstance(payload, ListFilesRequest):
            return {"ListFiles": payload}
        elif isinstance(payload, ReadFileRequest):
            return {"ReadFile": payload}
        else:
            return payload

    @field_validator("payload", mode="before")
    @classmethod
    def destructure_payload(cls, v):
        if "SystemTime" in v:
            return SystemTimeResponse(**v["SystemTime"])
        elif "ListFiles" in v:
            return ListFilesResponse(**v["ListFiles"])
        elif "ReadFile" in v:
            return ReadFileResponse(**v["ReadFile"])
        else:
            return v
