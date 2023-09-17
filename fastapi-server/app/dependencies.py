import uuid
from typing import Dict, Optional

from app.ws.session import WsSession
from fastapi import Depends, HTTPException, Request
from pydantic import BaseModel


async def conversation_header(request: Request) -> Optional[uuid.UUID]:
    return request.headers.get("openai-conversation-id")


class Conversation:
    def __init__(self, conv_id: uuid.UUID, session: WsSession):
        self.id = conv_id
        self.session = session


CONVERSATION_MAP: Dict[uuid.UUID, Conversation] = {}


async def get_conversation(
    conversation_id: Optional[uuid.UUID] = Depends(conversation_header),
) -> Conversation:
    if not conversation_id:
        raise HTTPException(
            status_code=400, detail="missing openai-conversation-id header"
        )
    if conversation_id not in CONVERSATION_MAP:
        raise HTTPException(
            status_code=404, detail="No Agent set for this conversation"
        )
    return CONVERSATION_MAP[conversation_id]
