"""
Noepedia API Client — Python interface to the Noepedia knowledge system.

Operations:
- Create publication
- Submit deltas (claims, evidence, sources)
- Query claims
- Export consolidated view
"""

import json
from dataclasses import dataclass, field
from typing import Optional
from uuid import UUID, uuid4

import httpx


@dataclass
class ClaimData:
    """A knowledge claim."""
    subject: str
    predicate: str
    object: str
    status: str = "PROPOSED"
    sources: list[UUID] = field(default_factory=list)
    evidence: list[UUID] = field(default_factory=list)
    claim_id: UUID = field(default_factory=uuid4)


@dataclass
class SourceData:
    """A source for a claim."""
    title: str
    source_type: str = "journal_article"
    url: Optional[str] = None
    doi: Optional[str] = None
    pmid: Optional[str] = None
    source_id: UUID = field(default_factory=uuid4)


class NoepediaClient:
    """Client for Noepedia knowledge system."""

    def __init__(self, api_url: str = "http://localhost:4000/api"):
        self.api_url = api_url.rstrip("/")
        self._client = httpx.Client(timeout=30.0)

    def create_publication(self, title: str, description: str = "") -> dict:
        """Create a new knowledge publication."""
        response = self._client.post(
            f"{self.api_url}/publications",
            json={"title": title, "description": description},
        )
        response.raise_for_status()
        return response.json()

    def submit_claim(self, publication_id: UUID, claim: ClaimData) -> dict:
        """Submit a claim as a delta to a publication."""
        response = self._client.post(
            f"{self.api_url}/publications/{publication_id}/claims",
            json={
                "claim_id": str(claim.claim_id),
                "subject": claim.subject,
                "predicate": claim.predicate,
                "object": claim.object,
                "status": claim.status,
                "sources": [str(s) for s in claim.sources],
                "evidence": [str(e) for e in claim.evidence],
            },
        )
        response.raise_for_status()
        return response.json()

    def query_claims(self, publication_id: UUID, status: Optional[str] = None) -> list[dict]:
        """Query claims in a publication, optionally filtered by status."""
        params = {}
        if status:
            params["status"] = status
        response = self._client.get(
            f"{self.api_url}/publications/{publication_id}/claims",
            params=params,
        )
        response.raise_for_status()
        return response.json()

    def get_consolidated_view(self, publication_id: UUID) -> dict:
        """Get the consolidated view of a publication."""
        response = self._client.get(
            f"{self.api_url}/publications/{publication_id}/consolidated"
        )
        response.raise_for_status()
        return response.json()

    def search(self, query: str) -> list[dict]:
        """Search across publications."""
        response = self._client.get(
            f"{self.api_url}/search", params={"q": query}
        )
        response.raise_for_status()
        return response.json()

    def close(self):
        self._client.close()
