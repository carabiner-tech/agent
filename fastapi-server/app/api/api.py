import uuid
from typing import Union

from app.dependencies import (
    CONVERSATION_MAP,
    Conversation,
    conversation_header,
    get_conversation,
)
from app.rpc import (
    ListFilesRequest,
    ListFilesResponse,
    ReadFileRequest,
    ReadFileResponse,
    RpcError,
)
from app.ws.manager import WsSessionManager
from fastapi import APIRouter, Depends, HTTPException
from fastapi.responses import PlainTextResponse

router = APIRouter(prefix="/api")


@router.post("/use_agent/{agent_id}", operation_id="use_agent")
async def use_agent(
    agent_id: uuid.UUID,
    session_manager: WsSessionManager = Depends(WsSessionManager.instance),
    conversation_id: str = Depends(conversation_header),
) -> PlainTextResponse:
    print(agent_id)
    if not conversation_id:
        raise HTTPException(status_code=400, detail="Missing conversation ID")
    session = await session_manager.get_session(agent_id)
    if not session:
        raise HTTPException(status_code=404, detail="Agent not connected")
    CONVERSATION_MAP[conversation_id] = Conversation(conversation_id, session)
    return PlainTextResponse("Agent set for this conversation")


@router.post("/list_files", operation_id="list_files")
async def list_files(
    req: ListFilesRequest,
    conv: Conversation = Depends(get_conversation),
) -> Union[ListFilesResponse, RpcError]:
    """RPC operation to list files in the current working directory or subdirectory for the active Agent"""
    return await conv.session.send_rpc(req)


@router.post("/read_file", operation_id="read_file")
async def read_file(
    req: ReadFileRequest,
    conv: Conversation = Depends(get_conversation),
) -> Union[ReadFileResponse, RpcError]:
    """RPC operation to read the content of a file at a path on the Agents system"""
    return await conv.session.send_rpc(req)
