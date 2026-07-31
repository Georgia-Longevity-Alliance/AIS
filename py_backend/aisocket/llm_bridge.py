"""
LLM Bridge — standard interface for any LLM to interact with AISocket.

Provides:
- Standard system prompt for AISocket context
- Tool definitions for passport reading, flight recorder access, trace recording
- Provider adapters (OpenAI, Gemini, Anthropic, Ollama)
"""

import json
from dataclasses import dataclass
from typing import Any, Optional, Protocol

AISOCKET_SYSTEM_PROMPT = """You are an AISocket-compatible diagnostic agent.

You have been invited to help a device that has encountered a problem it cannot
resolve deterministically.

CAPABILITIES:
- read_passport(device_name) — Read the device's passport
- read_flight_recorder(device_name, n_events) — Read recent flight recorder events
- diagnose(observation) — Submit a diagnosis
- act(capability, parameters) — Execute an action within the device's mandate
- record_trace(diagnosis, actions, outcome) — Record this intervention

RULES:
1. ALWAYS read the passport FIRST before suggesting any action.
2. NEVER suggest an action in the forbidden_always list.
3. If the flight recorder shows a known pattern, reference prior traces.
4. If uncertain, state your confidence level and ask clarifying questions.
5. After acting, ALWAYS record a trace.

FORMAT: Respond with structured JSON when invoking tools.
"""

DIAGNOSTIC_PROMPT_TEMPLATE = """Device: {device_name}
Status: The device has reported BLOCKED state.

Flight Recorder (last {n_events} events):
{flight_log}

Passport:
{passport}

Task: Diagnose what went wrong and propose a safe action.
If you are confident, invoke act(). Otherwise, ask questions.
"""


@dataclass
class LLMResponse:
    """Structured response from an LLM."""
    text: str
    tool_calls: list[dict] = None
    model: str = ""
    tokens_used: int = 0

    def __post_init__(self):
        if self.tool_calls is None:
            self.tool_calls = []


class LLMProvider(Protocol):
    """Protocol for LLM providers."""

    def chat(self, messages: list[dict], tools: Optional[list[dict]] = None) -> LLMResponse:
        ...


class OpenAIProvider:
    """OpenAI-compatible provider (OpenAI, local Ollama, etc.)."""

    def __init__(self, api_key: str, model: str = "gpt-4o", base_url: Optional[str] = None):
        try:
            from openai import OpenAI
        except ImportError:
            raise ImportError("openai package required. pip install openai")
        self.client = OpenAI(api_key=api_key, base_url=base_url)
        self.model = model

    def chat(self, messages: list[dict], tools: Optional[list[dict]] = None) -> LLMResponse:
        kwargs = {"model": self.model, "messages": messages}
        if tools:
            kwargs["tools"] = tools

        response = self.client.chat.completions.create(**kwargs)
        choice = response.choices[0]

        tool_calls = []
        if choice.message.tool_calls:
            for tc in choice.message.tool_calls:
                tool_calls.append({
                    "id": tc.id,
                    "name": tc.function.name,
                    "arguments": json.loads(tc.function.arguments),
                })

        return LLMResponse(
            text=choice.message.content or "",
            tool_calls=tool_calls,
            model=self.model,
            tokens_used=response.usage.total_tokens if response.usage else 0,
        )


class GeminiProvider:
    """Google Gemini provider."""

    def __init__(self, api_key: str, model: str = "gemini-2.0-flash"):
        try:
            import google.generativeai as genai
        except ImportError:
            raise ImportError("google-generativeai required. pip install google-generativeai")
        genai.configure(api_key=api_key)
        self.model = genai.GenerativeModel(model)
        self.model_name = model

    def chat(self, messages: list[dict], tools: Optional[list[dict]] = None) -> LLMResponse:
        # Convert OpenAI-style messages to Gemini format
        gemini_messages = []
        for msg in messages:
            role = "user" if msg["role"] == "user" else "model"
            gemini_messages.append({"role": role, "parts": [msg["content"]]})

        response = self.model.generate_content(gemini_messages)
        return LLMResponse(
            text=response.text,
            model=self.model_name,
            tokens_used=0,  # Gemini doesn't always report token counts
        )


class LLMBridge:
    """Bridge between AISocket and any LLM provider."""

    AISOCKET_TOOLS = [
        {
            "type": "function",
            "function": {
                "name": "read_passport",
                "description": "Read a device's passport — capabilities, forbidden actions, mandate",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "device_name": {"type": "string", "description": "Device name to look up"}
                    },
                    "required": ["device_name"],
                },
            },
        },
        {
            "type": "function",
            "function": {
                "name": "read_flight_recorder",
                "description": "Read recent events from a device's flight recorder",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "device_name": {"type": "string"},
                        "n_events": {"type": "integer", "default": 50},
                    },
                    "required": ["device_name"],
                },
            },
        },
        {
            "type": "function",
            "function": {
                "name": "diagnose",
                "description": "Submit a diagnosis for the current anomaly",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "diagnosis": {"type": "string"},
                        "confidence": {"type": "number", "minimum": 0, "maximum": 1},
                    },
                    "required": ["diagnosis", "confidence"],
                },
            },
        },
        {
            "type": "function",
            "function": {
                "name": "act",
                "description": "Execute a capability within the device's mandate",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "capability": {"type": "string"},
                        "parameters": {"type": "object"},
                    },
                    "required": ["capability"],
                },
            },
        },
        {
            "type": "function",
            "function": {
                "name": "record_trace",
                "description": "Record this intervention for the trace network",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "diagnosis": {"type": "string"},
                        "actions": {"type": "array", "items": {"type": "string"}},
                        "outcome": {"type": "string", "enum": ["resolved", "mitigated", "escalated", "unresolved"]},
                    },
                    "required": ["diagnosis", "outcome"],
                },
            },
        },
    ]

    def __init__(self, provider: LLMProvider):
        self.provider = provider

    def diagnose_anomaly(
        self,
        device_name: str,
        passport: dict,
        flight_log: str,
        n_events: int = 50,
    ) -> LLMResponse:
        """Ask the LLM to diagnose a device anomaly."""
        prompt = DIAGNOSTIC_PROMPT_TEMPLATE.format(
            device_name=device_name,
            passport=json.dumps(passport, indent=2),
            flight_log=flight_log,
            n_events=n_events,
        )

        messages = [
            {"role": "system", "content": AISOCKET_SYSTEM_PROMPT},
            {"role": "user", "content": prompt},
        ]

        return self.provider.chat(messages, tools=self.AISOCKET_TOOLS)
