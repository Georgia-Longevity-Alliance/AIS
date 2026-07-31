# MEMORY — AIS

**Created:** 2026-07-31

## 2026-07-31 — Project Creation

**Decision:** Created AIS as umbrella project under Marketing/ — merging AISocket + Noepedia into single implementation.

**Rationale:** 
- Gakely's AISocket and Noepedia are conceptually strong but lack code
- ARGUS-OS1 is the perfect first integration target
- Rust for core protocol (safety + performance), Python for AI/ML, Phoenix for web

**Key insights from code review:**
- AISocket: Python "Ready" but no Python code in repo — need to build it
- Noepedia: pure concept paper, no implementation — need MVP with 3-4 object types
- Both share architectural DNA: addressable, append-only, deterministic safety boundaries

**Next:**
1. Initialize Rust core with passport + body law
2. Autofix to 100
3. Python LLM bridge
4. Phoenix dashboard
5. ARGUS-OS1 integration demo
