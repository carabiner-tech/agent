import uuid

from app.dependencies import (
    CONVERSATION_MAP,
    Conversation,
    conversation_header,
    get_conversation,
)
from app.rpc import (
    ListFilesRequest,
    ListFilesResponse,
    SystemTimeRequest,
    SystemTimeResponse,
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


@router.post("/current_time", operation_id="current_time")
async def current_time(
    conv: Conversation = Depends(get_conversation),
) -> PlainTextResponse:
    """Show the Agents current system time"""
    req = SystemTimeRequest()
    resp: SystemTimeResponse = await conv.session.send_rpc(req)
    return PlainTextResponse(f"Current time is {resp.time}")


@router.post("/list_files", operation_id="list_files")
async def list_files(
    req: ListFilesRequest,
    conv: Conversation = Depends(get_conversation),
) -> PlainTextResponse:
    """List files in the current working directory or a subdirectory"""
    resp = await conv.session.send_rpc(req)
    text = ""
    for file in resp.files:
        text += f"{file.name} ({file.size} bytes)\n"
    for dir in resp.directories:
        text += f"{dir}/\n"
    return PlainTextResponse(text)
