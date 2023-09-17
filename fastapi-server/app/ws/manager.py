import uuid
from typing import Dict, Optional

from app.ws.session import WsSession


class WsSessionManager:
    _singleton_instance = None

    def __init__(self):
        self.sessions: Dict[uuid.UUID : WsSession] = {}

    @classmethod
    def instance(cls):
        if not cls._singleton_instance:
            cls._singleton_instance = WsSessionManager()
        return cls._singleton_instance

    async def get_session(self, id: uuid.UUID) -> Optional[WsSession]:
        return self.sessions.get(id, None)

    async def add_session(self, session: WsSession):
        self.sessions[session.id] = session

    async def remove_session(self, id: uuid.UUID):
        self.sessions.pop(id, None)
