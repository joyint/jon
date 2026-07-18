# Architecture

## Technology Stack

| Component     | Version | Rationale |
|---------------|---------|-----------|
| Rust          | 2021 edition | Same toolchain as joy/jyn; single static binaries |
| joy-core      | 0.19 (path dep) | The PDA writes projects, docs, and items through Joy's domain logic (JON-0005-19) |
| clap          | 4.5     | CLI parsing, same as joy-cli |
| minijinja     | 2.18    | PDA session template rendering, same engine as joy-core's AI templates (ADR-024 pattern) |
| serde_yaml_ng / serde_json | 0.10 / 1.0 | Embedded interview data, JSON subprocess contract |
| thiserror     | 2.0     | Typed jon-core errors wrapping JoyError |

## Repository Structure

```
jon/
├── crates/
│   ├── jon-core/           # Library: PDA session engine
│   │   ├── src/
│   │   │   ├── bootstrap.rs    # inspect + bootstrap (joy init, doc scaffolds, tool setup)
│   │   │   ├── pda.rs          # session prompt rendering
│   │   │   └── error.rs
│   │   ├── templates/pda/session.md
│   │   ├── data/pda/interview.yaml
│   │   └── data/docs/          # VISION/ARCHITECTURE/CONTRIBUTING scaffolds
│   ├── jon-cli/            # Binary `jon`: init (PDA bootstrap) + query entry
│   │   └── tests/init.rs   # e2e coverage for jon init
│   └── jon/                # Umbrella binary; TUI/desktop modes later
├── VISION.md               # Product vision (authoritative)
├── ARCHITECTURE.md         # This file
└── CONTRIBUTING.md
```

## Data Storage

Jon owns no data. The PDA writes into the target project's `.joy` store
and working tree through joy-core (documents, decision items, the first
task); the routing tiers read Joy/Jyn state over the `--json`
subprocess contract. Templates and interview data are embedded in the
binary; nothing is synced to disk at runtime.

## Architectural Decisions

- JON-0005-19: `jon-core` library extending `joy-core` (the jyn-core
  pattern), superseding the earlier subprocess-only rule for the PDA.
  The routing tiers (JON-0001-C1 rule engine, JON-000D-58 embedded LLM)
  stay subprocess-based.
- ADR-024 (joy): templates are embedded and rendered straight to the
  consumer; jon-core follows the same pattern for the PDA session.
- The PDA session prompt is rendered on demand and handed to the
  frontend (the CLI launches the local agent with it; the platform
  seeds the "Jon PDA" chat); it is not persisted by jon-core.

## Performance Targets

- CLI startup and Tier 0 routing: instant (no network, no model).
- PDA bootstrap: bounded by `git init` and file writes, well under a
  second on local disks; the conversation itself is bounded by the
  agent, not by Jon.
