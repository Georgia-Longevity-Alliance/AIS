# TODO — AIS

**Updated:** 2026-07-31

## Phase 0: Foundation (current)
- [ ] Create core Rust project (`core/Cargo.toml` + `src/lib.rs`)
- [ ] Implement Passport struct + serialization (serde)
- [ ] Implement Body Law — 6-layer validation pipeline
- [ ] Implement Flight Recorder — ring buffer
- [ ] Implement Trace Network — intervention trace struct
- [ ] Implement Delta Protocol — legal operations
- [ ] Implement Event Store — append-only log (SQLite)
- [ ] Implement Validator — schema + status transitions
- [ ] Create Python backend (`py_backend/`)
- [ ] Implement LLM Bridge — standard prompt + tool interface
- [ ] Implement HTTP Client — googuly.online + local registry
- [ ] Create Phoenix web project (`web/`)
- [ ] Implement Dashboard LiveView
- [ ] Implement Device Registry page
- [ ] Implement Knowledge Browser page
- [ ] Integration: ARGUS-OS1 Passport example
- [ ] Integration: Simulation test (virtual mower)

## Phase 1: ARGUS-OS1 Integration
- [ ] Deploy Flight Recorder on Jetson Orin NX
- [ ] Connect to googuly.online registry
- [ ] LLM anomaly diagnosis test (3 simulated failures)
- [ ] Trace Network: record + retrieve across sessions
- [ ] Noepedia: publish ARGUS-OS1 findings as knowledge field

## Phase 2: Production Hardening
- [ ] Cryptographic passport signatures (Ed25519)
- [ ] Federation between registry servers
- [ ] PostgreSQL event store for server deployment
- [ ] WebRTC direct device connection
- [ ] ROS2 bridge
- [ ] Matter/WoT compatibility layer

## Phase 3: Community
- [ ] Open source release (Apache 2.0 + AGPL for Noepedia)
- [ ] Documentation site
- [ ] Contributor guide
- [ ] Demo video
