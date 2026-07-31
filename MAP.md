# MAP — AIS

```
AIS/                           # Root: core files only
├── _pi.md                     # Rules for pi
├── CONCEPT.md                 # Concept + architecture
├── TODO.md                    # Task list
├── PARAMETERS.md              # Protocol + safety parameters
├── MAP.md                     # This file
├── STATE.md                   # Current status
├── MEMORY.md                  # Decision history
├── README.md                  # Public-facing overview
│
├── core/                      # Rust — protocol implementation
│   ├── Cargo.toml
│   ├── Cargo.lock
│   └── src/
│       ├── lib.rs             # Public API
│       ├── passport.rs        # Passport struct
│       ├── body_law.rs        # 6-layer validator
│       ├── flight_recorder.rs # Ring buffer
│       ├── trace.rs           # Intervention traces
│       ├── delta.rs           # Noepedia delta protocol
│       ├── event_store.rs     # Append-only log
│       ├── validator.rs       # Delta validator
│       ├── consolidator.rs    # Knowledge consolidation
│       ├── renderer.rs        # Article renderer
│       └── types.rs           # Shared types + serde
│
├── py_backend/                # Python — AI/ML + HTTP
│   ├── requirements.txt
│   ├── setup.py
│   ├── aisocket/
│   │   ├── __init__.py
│   │   ├── client.py          # Registry HTTP client
│   │   ├── llm_bridge.py      # LLM ↔ AISocket
│   │   ├── noepedia_api.py    # Noepedia client
│   │   ├── anomaly.py         # Anomaly detection
│   │   └── prompts.py         # Standard LLM prompts
│   └── tests/
│       ├── test_client.py
│       ├── test_llm_bridge.py
│       └── test_noepedia.py
│
├── web/                        # Elixir/Phoenix — Dashboard
│   ├── mix.exs
│   ├── mix.lock
│   ├── config/
│   ├── lib/
│   │   ├── ais_web.ex
│   │   ├── ais_web/
│   │   │   ├── endpoint.ex
│   │   │   ├── router.ex
│   │   │   ├── pages/
│   │   │   │   ├── dashboard_live.ex
│   │   │   │   ├── device_live.ex
│   │   │   │   └── knowledge_live.ex
│   │   │   └── components/
│   │   └── ais/
│   │       ├── registry.ex     # Device registry
│   │       ├── knowledge.ex    # Noepedia client
│   │       └── socket_client.ex # Rust core bridge
│   └── test/
│
├── integration/                # End-to-end tests
│   └── argus_os1/
│       ├── passport_argus.json # ARGUS-OS1 passport
│       ├── simulate_run.py     # 100-embryo simulation
│       └── test_anomalies.py   # Anomaly injection tests
│
├── docs/                       # Documentation
│   ├── ARCHITECTURE.md
│   ├── API_REFERENCE.md
│   ├── CONTRIBUTING.md
│   └── GLOSSARY.md
│
├── scripts/                    # Utility scripts
│   ├── autofix.sh
│   ├── setup_dev.sh
│   └── run_tests.sh
│
└── _archive/                   # Superseded files
```
