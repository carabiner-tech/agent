from functools import lru_cache

from app.settings import get_settings
from pydantic import BaseModel, Field

settings = get_settings()


class Api(BaseModel):
    type: str = "openapi"
    url: str = settings.openapi_json_url


class Auth(BaseModel):
    type: str = "none"


class Manifest(BaseModel):
    schema_version: str = "v1"
    name_for_human: str = "Carabiner Demo Server"
    name_for_model: str = "carabiner_demo_server"
    logo_url: str = settings.logo_url
    contact_email: str = "author@example.com"
    legal_info_url: str = "https://example.com/legal"
    api: Api = Field(default_factory=Api)
    auth: Auth = Field(default_factory=Auth)
    description_for_human: str = "Carabiner demo server"
    description_for_model: str = "Introspect the file system of a running Agent"


@lru_cache
def get_manifest() -> Manifest:
    return Manifest()
