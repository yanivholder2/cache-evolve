# Project Handbook (AGENTS.md)

## Research Context
M.Sc. thesis (Yaniv Holder, advised by Prof. Roy Friedman, Technion) on mitigating the **hot set problem** in N-way associative caches using Dash-inspired bucket movement. Items can migrate between adjacent buckets to reduce contention in overloaded sets. See `research-proposal.tex` for full proposal, motivation, and initial results.

## Source Control & Workflow
After completing a logical unit of work (feature, fix, refactor, etc.), propose a commit. Avoid staging/committing changes unless explicitly requested.

## Architecture
Cache simulator for thesis research. Integrates with Java's Caffeine cache simulator via JNI.

### JNI (`src/dash/dash_jni.rs`, `src/associative_cache/associative_cache_jni.rs`)
Bridges to Java Caffeine simulator. Types fixed to `i64`. Miss sentinel: `-1`.
- Raw pointer management via `Box::into_raw` / `Box::from_raw`
- Dash JNI logs to `rust-logs/`; Associative JNI has no logging
- Java classes: `com.github.benmanes.caffeine.cache.simulator.policy.{dash.DashRustPolicy, associative.AssociativeCacheRustPolicy}`

## Caffeine Simulator Integration
Dash cache integrates with [Caffeine](https://github.com/ben-manes/caffeine)'s cache simulator via JNI.

**Repo**: `C:\Users\yanivholder\Downloads\ThesisNew\caffeine` (the only caffeine repo to use)

**Java policy class**: `simulator/src/main/java/.../policy/dash/DashRustPolicy.java`
- Pointer-based JNI: stores `cachePointer` from `initDefaultCache()` / `initCache(...)`, passes to all calls
- `DashRustEvictionPolicy` enum ordinals must match Rust `from_usize` indices
- `DashSettings` inner class reads `dash.*` config from `application.conf`

**Build + Run workflow**:
1. `cargo build --release` (produces `target/release/dash.dll`)
2. From caffeine dir: `./gradlew simulator:run -PjvmArgs="-XX:+UseParallelGC,-Xmx8g,-Djava.library.path=<path-to-dash-cache-rs>/target/release" -q`

## Backlog (`backlog.md`)
Track bugs, tasks, and small TODOs.
- **Sections**: Bugs, Tasks, Small TODOs.
- **Format**: Each item gets a `###` heading with Created date and Priority (0-5).
- **Maintenance**: Update the TOC at the top of `backlog.md` when adding/removing items.