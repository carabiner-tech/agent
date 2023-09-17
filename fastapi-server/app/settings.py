from functools import lru_cache

from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    host: str = '127.0.0.1:8000'
    public_url: str = 'http://localhost:8000'

    @property
    def logo_url(self) -> str:
        return f'{self.public_url}/logo.png'
    
    @property
    def openapi_json_url(self) -> str:
        return f'{self.public_url}/openapi.json'
    
@lru_cache
def get_settings() -> Settings:
    return Settings()