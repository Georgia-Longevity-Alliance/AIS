"""
AISocket HTTP Client — connect to googuly.online registry.

Handles:
- Device registration
- Device lookup
- Heartbeat
- Passport upload/download
"""

import json
from dataclasses import dataclass, field
from typing import Optional
from uuid import UUID, uuid4

import httpx


@dataclass
class DeviceInfo:
    """A device registered in the AISocket network."""
    name: str
    ip: str
    port: int
    description: str = ""
    device_id: UUID = field(default_factory=uuid4)
    passport: Optional[dict] = None


class AISocketClient:
    """HTTP client for AISocket registry."""

    def __init__(self, registry_url: str = "https://googuly.online/aisocket"):
        self.registry_url = registry_url.rstrip("/")
        self._client = httpx.Client(timeout=10.0)

    def register(self, device: DeviceInfo) -> dict:
        """Register a device in the AISocket registry."""
        response = self._client.post(
            f"{self.registry_url}/register.php",
            json={
                "name": device.name,
                "ip": device.ip,
                "port": device.port,
                "device_id": str(device.device_id),
                "prompt": device.description,
                "passport": device.passport,
            },
        )
        response.raise_for_status()
        return response.json()

    def lookup(self, name: str) -> Optional[dict]:
        """Look up a device by name."""
        response = self._client.get(
            f"{self.registry_url}/device.php", params={"name": name}
        )
        if response.status_code == 404:
            return None
        response.raise_for_status()
        return response.json()

    def heartbeat(self, device_id: UUID) -> dict:
        """Send a heartbeat to keep registration alive."""
        response = self._client.post(
            f"{self.registry_url}/heartbeat.php",
            json={"device_id": str(device_id)},
        )
        response.raise_for_status()
        return response.json()

    def upload_passport(self, device_id: UUID, passport: dict) -> dict:
        """Upload or update a device passport."""
        response = self._client.post(
            f"{self.registry_url}/passport.php",
            json={"device_id": str(device_id), "passport": passport},
        )
        response.raise_for_status()
        return response.json()

    def close(self):
        self._client.close()
