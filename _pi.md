# _pi.md — AIS (AISocket + Noepedia Implementation)

**Project:** AIS — Autonomous Intelligence Socket
**Parent:** ~/Desktop/Marketing/
**Created:** 2026-07-31

## Rules for pi

1. **Rust first.** Core protocol, Body Law, Flight Recorder, Event Store, Delta Validator — на Rust.
2. **Python fallback.** HTTP-клиенты, AI/ML-интеграции, научные расчёты — на Python.
3. **Phoenix web.** Дашборд, LiveView, API Gateway — на Elixir/Phoenix.
4. **Autofix cycles.** После каждого значимого изменения — autofix. Цель: 100/100.
5. **Commit after autofix.** Каждый цикл → git commit.
6. **Язык общения.** Строго на русском. Код и комментарии — английский.

## Project Structure
```
AIS/
├── core/           # Rust — AISocket protocol core
│   ├── Cargo.toml
│   └── src/
│       ├── passport.rs      # Passport struct + validation
│       ├── body_law.rs      # 6-layer command validation
│       ├── flight_recorder.rs # Ring buffer for device events
│       ├── trace.rs         # Trace Network protocol
│       ├── delta.rs         # Delta Protocol (Noepedia)
│       ├── event_store.rs   # Append-only event log
│       ├── validator.rs     # Delta validator
│       └── lib.rs
├── py_backend/     # Python — AI/ML + HTTP integration
│   ├── requirements.txt
│   ├── aisocket/
│   │   ├── __init__.py
│   │   ├── client.py        # HTTP client for AISocket registry
│   │   ├── llm_bridge.py    # LLM ↔ AISocket interface
│   │   └── noepedia_api.py  # Noepedia API client
│   └── tests/
├── web/            # Elixir/Phoenix — Dashboard
│   ├── mix.exs
│   └── lib/
│       └── ais_web/
├── docs/           # Documentation
├── scripts/        # Utility scripts
├── integration/    # Integration tests + examples
│   └── argus_os1/  # ARGUS-OS1 integration example
└── _archive/       # Archived files
```

## Autofix Command
```bash
cd ~/Desktop/Marketing/AIS && bash ~/Desktop/Services/scripts/autofix.sh AIS 100
```
