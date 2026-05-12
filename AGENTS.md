# Research Context
M.Sc. thesis on mitigating the **hot set problem** in N-way associative caches using Dash-inspired bucket movement. Items can migrate between adjacent buckets to reduce contention in overloaded sets. See `research-proposal.tex` for full proposal, motivation, and initial results.


# Source Control & Workflow
After completing a logical unit of work (feature, fix, refactor, etc.), propose a commit. Avoid staging/committing changes unless explicitly requested.
All new worktrees must be created inside the `.worktrees/` directory in the repository root.

# Conventions

See `../docs/CONVENTIONS.md`

# Caffeine Simulator Integration
Dash cache integrates with [Caffeine](https://github.com/ben-manes/caffeine)'s cache simulator via JNI.

**Repo**: `../caffeine` (sibling of this repo; the only caffeine repo to use)

**Invariants**:
- Pointer-based JNI: Java side stores the pointer returned from cache init and passes it back on every call.
- Eviction-policy enum ordinals must match Rust `EvictionPolicy::from_usize` indices.
- Java settings inner classes read `dash.*` config from `application.conf`.

**Build + Run workflow**:
1. `cargo build --release` (produces `target/release/dash.dll`)
2. `cargo test`
3. (Optional, only if requested) From caffeine dir: `./gradlew simulator:run -PjvmArgs="-XX:+UseParallelGC,-Xmx8g,-Djava.library.path=<path-to-dash-cache-rs>/target/release" -q`

# Agent skills

## Issue tracker

Issues live in GitHub Issues (`gh` CLI). See `docs/agents/issue-tracker.md`.

## Triage labels

Default label vocabulary is in `docs/agents/triage-labels.md`.

## Domain docs

Single-context layout — one `CONTEXT.md` + `docs/adr/` at the repo root. See `docs/agents/domain.md`.
