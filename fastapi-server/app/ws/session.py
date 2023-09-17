import asyncio
import uuid
from typing import Dict, Optional

from app.rpc import Message, RpcRequest, RpcResponse
from fastapi import WebSocket
from pydantic import ValidationError


class WsSession:
    def __init__(self, conn: WebSocket):
        self.id = uuid.uuid4()
        self.conn = conn
        self.callbacks: Dict[uuid.UUID: asyncio.Future] = {}

    async def handle_message(self, msg: str):
        print(f"Received message: {msg}")
        try:
            parsed = Message.model_validate_json(msg)
        except ValidationError as e:
            print(f"Failed parsing to Message: {e}")
            return
        except Exception as e:
            print(f"Unexpected error: {e}")
            return
        cb: Optional[asyncio.Future] = self.callbacks.pop(parsed.id, None)
        if not cb:
            print(f"Received message with no callback: {parsed}")
            return
        cb.set_result(parsed.payload)

    async def send(self, text: str):
        print(f"Sending message: {text}")
        await self.conn.send_text(text)

    async def send_rpc(self, req: RpcRequest) -> RpcResponse:
        msg_id = uuid.uuid4()
        msg = Message(id=msg_id, payload=req)
        fut = asyncio.Future()
        self.callbacks[msg_id] = fut
        await self.send(msg.model_dump_json())
        return await fut



        