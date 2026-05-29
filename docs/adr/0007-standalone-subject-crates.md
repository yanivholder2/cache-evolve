# Standalone subject crates with manifest-driven wiring

Each Subject is its own Rust crate (its own `Cargo.toml`, cdylib, and copy of `src/shared/`) under `inputs/subjects/<name>/`, depending only on the `framework` crate — rather than living as modules in one shared `dash` crate. All subject-specific integration knowledge (Caffeine config prefix, policy spec name, geometry, policy-list key, protected files, correctness gates via `[[gates]]`, optional microbench) is declared in each subject's `manifest.toml` and read generically by the framework, so `caffeine_runner.py` and the rest of the framework name no subject.

## Considered Options

- **One crate, modules per subject** (the prior shape): less duplication, but a Full-autonomy variant rewriting one subject could touch sibling subjects, and the framework hardcoded per-subject wiring (e.g. `caffeine_runner.py`'s `TARGETS` dict).
- **A shared library crate** for `src/shared/`: removes duplication, but recouples subjects — a variant can no longer freely rewrite shared code without affecting siblings.

## Consequences

- `src/shared/` is duplicated per subject by design; the isolation is the point.
- Adding a subject is a self-contained operation: new crate + manifest, no framework edits. A subject declares its own correctness gates (`[[gates]]`) and `tests/` scaffolds the same way — the framework gates (build, contract, universal invariant) run first, then the subject's.
- Subjects with zero extension points (e.g. `lru_cache`) are valid; their variants run in Full-autonomy scope only.
