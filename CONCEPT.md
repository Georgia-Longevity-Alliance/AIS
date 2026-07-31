# CONCEPT — AIS (Autonomous Intelligence Socket)

**Version:** 1.0  
**Date:** 2026-07-31  
**Parent:** ~/Desktop/Marketing/

---

## 0. Executive Summary

AIS is the reference implementation merging **AISocket** (embodied AI safety protocol) and **Noepedia** (hallucination-resistant knowledge system) into a single, production-grade platform.

**Three pillars:**
1. **AISocket Core** (Rust) — Passport, Body Law, Flight Recorder, Trace Network
2. **Noepedia Core** (Rust) — Delta Protocol, Event Store, Validator, Consolidator
3. **AIS Web** (Phoenix/Elixir) — Dashboard, Device Registry, Knowledge Browser, LiveView

**Mission:** Give every autonomous body a passport, every intervention a trace, every knowledge claim a provenance — and make it all open-source, energy-honest, and safe.

---

## 1. Why AIS Exists

### 1.1 The Two Missing Pieces

Current AI landscape has two structural gaps:

| Gap | Problem | AIS Solution |
|-----|---------|--------------|
| **Embodied safety** | LLMs can't safely control physical devices — safety rules in prompts are unreliable | Body Law in firmware, `forbidden_always` enforced at ALU cost |
| **Knowledge hallucination** | LLM answers lack provenance — `PROPOSED` masked as `FACT`, conflicts disappear in fluent text | Structured knowledge field: claims → sources → evidence → status |

AISocket + Noepedia were born as separate projects. AIS unifies them because they share the same architectural DNA: **addressable, verifiable, append-only structures with deterministic safety boundaries.**

### 1.2 The Energy Argument

```
LLM token: ~1 joule
Microcontroller addition: ~1 picojoule
Ratio: 10¹² : 1
```

Using an LLM as a continuous controller is like running to the equator to take a single step. AIS enforces:
- **Deterministic execution** (firmware, SQL, graph traversal) for routine operations
- **LLM intervention** only for novelty, conflict, and open questions
- **Trace preservation** so solved problems are never paid for twice

---

## 2. Architecture

### 2.1 Layer Model

```
┌─────────────────────────────────────────────┐
│              WEB LAYER (Phoenix)             │
│  Dashboard │ Registry │ Knowledge Browser    │
├─────────────────────────────────────────────┤
│            PYTHON LAYER (py_backend)         │
│  LLM Bridge │ HTTP Client │ ML Integration   │
├─────────────────────────────────────────────┤
│              RUST CORE (core/)               │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │ AISocket  │  │ Noepedia │  │  Shared  │   │
│  │ Passport  │  │  Delta   │  │  Event   │   │
│  │ Body Law  │  │Validator │  │  Store   │   │
│  │ FlightRec │  │Consolid. │  │  Types   │   │
│  │ TraceNet  │  │ Renderer │  │  Crypto  │   │
│  └──────────┘  └──────────┘  └──────────┘   │
├─────────────────────────────────────────────┤
│           DEVICE LAYER (firmware)            │
│  ESP32 │ Arduino │ Raspberry Pi │ Jetson     │
└─────────────────────────────────────────────┘
```

### 2.2 Data Flow

```
DEVICE                    AIS CORE                   USER/LLM
  │                          │                          │
  ├─ heartbeat ─────────────→│                          │
  │                          ├─ registry update         │
  │                          │                          │
  ├─ event ─────────────────→│                          │
  │                          ├─ flight_recorder.write() │
  │                          │                          │
  │                    ┌─────┤                          │
  │                    │BLOCKED?                        │
  │                    └─────┤                          │
  │                          ├─ LLM invoked ←───────────┤
  │                          ├─ reads passport          │
  │                          ├─ reads flight recorder   │
  │                          ├─ diagnoses               │
  │                          ├─ acts (within mandate)   │
  │                          ├─ writes trace            │
  │                          │                          │
  │                          ├─ knowledge → Noepedia ───→│
  │                          │                          │
```

---

## 3. Component Specifications

### 3.1 AISocket Core (Rust)

#### Passport (`core/src/passport.rs`)
```rust
struct Passport {
    device_id: Uuid,
    name: String,
    description: String,
    capabilities: Vec<Capability>,
    forbidden_always: Vec<ForbiddenAction>,  // Enforced in firmware
    risk_class: RiskClass,
    autonomous_mandate: Option<AutonomousMandate>,
    emergency_contacts: Vec<EmergencyContact>,
    platform: Platform,
    version: SemVer,
    signature: Option<Ed25519Signature>,     // Roadmap: cryptographic
}
```

#### Body Law (`core/src/body_law.rs`)
6-layer validation pipeline:
1. **Firmware rules** — hardware-enforced constraints (torque, temp, laser power)
2. **Capability check** — is this action in the passport?
3. **Emergency override** — rescue agents with limited access
4. **Offline mandate** — what's allowed when connection drops
5. **Delegation chain** — who authorized whom
6. **Context validation** — time, environment, state preconditions

#### Flight Recorder (`core/src/flight_recorder.rs`)
```rust
struct FlightRecorder {
    buffer: RingBuffer<FlightEvent>,
    capacity: usize,  // ESP32: 256, Android: 65536
    event_types: HashSet<EventType>,
}
```

#### Trace Network (`core/src/trace.rs`)
```rust
struct InterventionTrace {
    trace_id: Uuid,
    device_id: Uuid,
    agent_id: Uuid,
    diagnosis: String,
    actions_taken: Vec<Action>,
    outcome: Outcome,
    timestamp: DateTime<Utc>,
    references: Vec<Uuid>,  // Links to prior related traces
}
```

### 3.2 Noepedia Core (Rust)

#### Delta Protocol (`core/src/delta.rs`)
Legal operations: CREATE, UPDATE, RELATE, DEPRECATE, CONSOLIDATE.
Every delta is a proposed change — the validator decides acceptance.

#### Event Store (`core/src/event_store.rs`)
Append-only log. SQLite for embedded, PostgreSQL for server.
`(event_id, timestamp, actor, operation, payload, signature)`

#### Validator (`core/src/validator.rs`)
Checks: schema conformance, referential integrity, status transitions, permission boundaries, consistency rules.

### 3.3 Python Backend (`py_backend/`)

- **LLM Bridge:** Standard prompt + tool interface for any LLM (OpenAI, Anthropic, Gemini, local Ollama)
- **HTTP Client:** Connect to googuly.online registry + Noepedia API
- **Scientific:** NumPy/SciPy for flight recorder analysis, scikit-learn for anomaly detection

### 3.4 Phoenix Web (`web/`)

- **Dashboard:** Device status, recent traces, active sessions
- **Registry:** Device search, passport viewer, permission management
- **Knowledge Browser:** Noepedia publication explorer, claim graph visualisation
- **LiveView:** Real-time flight recorder streaming

---

## 4. First MVP — ARGUS-OS1 Integration

The first real-world deployment target is **ARGUS-OS1** (automated centriole tracking microscope).

| AIS Component | ARGUS-OS1 Instantiation |
|---------------|------------------------|
| Passport | Microscope identity: 488/561/640nm lasers, Sangaboard stage, microfluidic |
| Body Law | Laser safety, temp ≤37°C, phototoxicity ceiling (div rate >90%) |
| Flight Recorder | Centriole tracking log: coordinates, intensities, division events |
| LLM Bridge | Anomaly detection → Gemini Flash diagnosis → safe restart or pause |
| Trace Network | Solved anomalies recorded → all ARGUS devices learn |
| Knowledge Field | Published findings → Noepedia claims with provenance |

---

## 5. Success Criteria

1. ✅ Passport generated for ARGUS-OS1 V6
2. ✅ Flight Recorder captures 100-embryo run
3. ✅ Body Law prevents laser over-power in firmware
4. ✅ LLM successfully diagnoses 3 simulated anomalies
5. ✅ Trace recorded and retrievable by second device
6. ✅ Noepedia publication created with ≥5 claims, each with source + evidence
7. ✅ Phoenix dashboard shows live device status

---

## 6. References

- AISocket: https://github.com/gakelytemp-creator/AISocket
- Noepedia: https://github.com/gakelytemp-creator/Noepedia
- ARGUS-OS1: ~/Desktop/Marketing/ARGUS-OS1/
- MCP Spec: https://modelcontextprotocol.io/
