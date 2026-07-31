# PARAMETERS — AIS

**Updated:** 2026-07-31

## Protocol Parameters

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| Flight Recorder buffer (ESP32) | 256 events | ~2KB RAM budget |
| Flight Recorder buffer (Jetson/RPi) | 65536 events | Full run trace |
| Heartbeat interval | 5s | Registry liveness detection |
| Body Law validation layers | 6 | firmware → capabilities → emergency → offline → delegation → context |
| Max passport size (ESP32) | 512B | JSON+signature fits in RAM |
| Max passport size (server) | 64KB | Rich layered documents |
| Delta types | 5 | CREATE, UPDATE, RELATE, DEPRECATE, CONSOLIDATE |
| Object types (Noepedia v1) | 5 | CLAIM, SOURCE, EVIDENCE, OPEN, CONFLICT |
| Event store batch size | 100 | SQLite pragma optimization |
| LLM max context tokens | 128K | Flight recorder + passport + prompt |
| LLM temperature (diagnosis) | 0.1 | Low: deterministic diagnosis |
| LLM temperature (exploration) | 0.7 | Creative problem-solving |

## Safety Parameters

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| Max laser power (ARGUS-OS1) | 10mW | Eye safety + embryo safety |
| Max temperature | 37°C | C. elegans lethal limit |
| Phototoxicity ceiling | 90% division rate | Pilot P2 Go/No-Go |
| `forbidden_always` override | NEVER | Constitutional invariant |
| Emergency mandate duration | 300s | Auto-expire rescue access |
| Offline mandate max duration | 12h | ARGUS overnight run |

## Performance Parameters

| Parameter | Target | Measurement |
|-----------|--------|-------------|
| Passport validation latency | <1ms | Rust, in-memory |
| Body Law check (6 layers) | <100μs | ALU-level |
| Flight Recorder write | <10μs | Ring buffer push |
| Event Store append | <1ms | SQLite WAL mode |
| Delta validation | <5ms | Referential integrity |
| LLM diagnosis latency | <2s | Gemini Flash |
| Dashboard LiveView latency | <50ms | Phoenix channels |
