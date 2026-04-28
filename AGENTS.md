# Research Context
M.Sc. thesis (Yaniv Holder, advised by Prof. Roy Friedman, Technion) on mitigating the **hot set problem** in N-way associative caches using Dash-inspired bucket movement. Items can migrate between adjacent buckets to reduce contention in overloaded sets. See `research-proposal.tex` for full proposal, motivation, and initial results.

# Domain Vocabulary
Canonical glossary of domain terms (Segment, Bucket, Stash, Target, Probing, Slot, Promote, Demote, Cascade, etc.) plus flagged ambiguities and their resolutions in [`UBIQUITOUS_LANGUAGE.md`](./UBIQUITOUS_LANGUAGE.md). Consult before naming new types/functions or when a term feels overloaded.

# Source Control & Workflow
After completing a logical unit of work (feature, fix, refactor, etc.), propose a commit. Avoid staging/committing changes unless explicitly requested.

# Architecture
Cache simulator for thesis research. Integrates with Java's Caffeine cache simulator via JNI.

## JNI
Bridges to Java Caffeine simulator. Types fixed to `i64`. Miss sentinel: `-1`.
- Raw pointer management via `Box::into_raw` / `Box::from_raw`
- All JNI modules log to `rust-logs/` via the shared logger

# Conventions

@docs/CONVENTIONS.md

# Caffeine Simulator Integration
Dash cache integrates with [Caffeine](https://github.com/ben-manes/caffeine)'s cache simulator via JNI.

**Repo**: `C:\Users\yanivholder\Downloads\ThesisNew\caffeine` (the only caffeine repo to use)

**Invariants**:
- Pointer-based JNI: Java side stores the pointer returned from cache init and passes it back on every call.
- Eviction-policy enum ordinals must match Rust `EvictionPolicy::from_usize` indices.
- Java settings inner classes read `dash.*` config from `application.conf`.

**Build + Run workflow**:
1. `cargo build --release` (produces `target/release/dash.dll`)
2. From caffeine dir: `./gradlew simulator:run -PjvmArgs="-XX:+UseParallelGC,-Xmx8g,-Djava.library.path=<path-to-dash-cache-rs>/target/release" -q`
