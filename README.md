# AIS — Autonomous Intelligence Socket

**Open-source protocol for safe embodied AI + hallucination-resistant knowledge.**

AIS gives every autonomous device a **passport** — so any LLM can safely understand it, help it, and leave a trace that every other machine learns from.

Built on two complementary protocols:
- **AISocket** — Safe AI control of physical devices (Body Law, Flight Recorder, Trace Network)
- **Noepedia** — Structured knowledge with provenance (claims → sources → evidence → status)

## Quick Start

```bash
# Rust core
cd core && cargo build

# Python backend
cd py_backend && pip install -e .

# Phoenix web
cd web && mix setup && mix phx.server
```

## Architecture

| Layer | Language | Role |
|-------|----------|------|
| Core | Rust | Passport, Body Law, Flight Recorder, Event Store |
| Backend | Python | LLM bridge, HTTP client, ML integration |
| Web | Elixir/Phoenix | Dashboard, Registry, Knowledge Browser |

## License

Core (Rust): Apache 2.0  
Noepedia components: AGPL v3.0  
Web: MIT

---

*Born from AISocket + Noepedia. Built for ARGUS-OS1.*
