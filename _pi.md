# _pi.md — AIS (Autonomous Intelligence Socket)

**Project:** AIS — Autonomous Intelligence Socket
**Parent:** ~/Desktop/Marketing/
**Created:** 2026-07-31

## Rules for pi

1. **Rust first.** Core protocol, Body Law, Flight Recorder, Event Store, Delta Validator — in Rust.
2. **Python fallback.** HTTP clients, AI/ML integrations, scientific calculations — in Python.
3. **Phoenix web.** Dashboard, LiveView, API Gateway — in Elixir/Phoenix.
4. **Autofix cycles.** After every significant change — autofix. Target: 100/100.
5. **Commit after autofix.** Each cycle → git commit.
6. **Language.** Communication in Russian. Code and comments in English.

## Project Structure
```
AIS/
├── core/           # Rust — AISocket protocol core
│   ├── Cargo.toml
│   └── src/
│       ├── passport.rs
│       ├── body_law.rs
│       ├── flight_recorder.rs
│       ├── trace.rs
│       ├── delta.rs
│       ├── event_store.rs
│       ├── validator.rs
│       ├── consolidator.rs
│       ├── renderer.rs
│       ├── types.rs
│       └── lib.rs
├── py_backend/     # Python — AI/ML + HTTP integration
│   ├── requirements.txt
│   └── aisocket/
│       ├── __init__.py
│       ├── client.py
│       ├── llm_bridge.py
│       └── noepedia_api.py
├── web/            # Elixir/Phoenix — Dashboard
│   ├── mix.exs
│   └── lib/
│       └── web_web/
│           ├── router.ex
│           ├── live/
│           │   ├── dashboard_live.ex
│           │   └── device_live.ex
│           └── controllers/
│               └── api_controller.ex
├── docs/           # Documentation
├── scripts/        # Utility scripts
├── integration/    # Integration tests + ARGUS-OS1 example
└── _archive/       # Reference implementations
```

## Autofix Command
```bash
cd ~/Desktop/Marketing/AIS && python3 ~/Desktop/Services/scripts/autofix.sh .
```
