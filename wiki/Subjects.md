# Subjects

A **subject** is the cache implementation you evolve. Evolve is subject-agnostic: the framework code knows nothing about any particular cache. Everything subject-specific is declared in the subject's `manifest.toml` and lives in its own crate.

## What a subject declares

Each subject is a self-contained Rust crate under `inputs/subjects/<name>/`, with a `manifest.toml` that declares:

- **Identity** — the crate, and the name of the native library it builds.
- **Extension points** — the code seams a variant is allowed to edit (see [Concepts](Concepts#extension-points-and-authoring-scope)).
- **Gates** — any subject-specific correctness tests, run after the framework gates.
- **Protected files** — files a variant can never overlay (e.g. the JNI boundary).
- **Simulator wiring** — the `[caffeine]` table (see [Evaluation and Fitness](Evaluation-and-Fitness#caffeine)).
- **Evaluation** — the baseline fitness anchor, plus the traces and cache sizes to measure over.
- **Docs** — the subject's own glossary and baseline insight.
- **Optional plugin hooks** — `[bench]` and `[metadata]` tables that switch on extra fitness dimensions.

## Included subjects

| Subject             | What it is                                              | Editable seams                         |
| ------------------- | ------------------------------------------------------ | -------------------------------------- |
| `dash`              | Multi-segment associative cache with bucket migration  | eviction-policy, bucket, item, segment |
| `associative_cache` | N-way associative baseline                             | eviction-policy, bucket                |
| `lru_cache`         | Classic LRU baseline                                   | none (capacity is the only knob)       |

Each subject keeps its own glossary at `inputs/subjects/<name>/CONTEXT.md`.

## Selecting a subject

Pick the subject with `--subject <name>`, or set `subject` in [`config.toml`](https://github.com/yanivholder2/cache-evolve/blob/main/config.toml). The name is the directory under `inputs/subjects/`. Everything the run touches — the manifest, the crate, the baseline — is resolved from that one choice.

## How subjects are structured

Each subject is a **standalone crate** that depends on a shared framework crate. This isolation is a deliberate design decision so that subjects don't entangle each other; see [ADR 0007](https://github.com/yanivholder2/cache-evolve/blob/main/docs/adr/0007-standalone-subject-crates.md). The single generic seam that lets the framework drive any subject is described in [ADR 0005](https://github.com/yanivholder2/cache-evolve/blob/main/docs/adr/0005-single-generic-seam.md).

## Going deeper

- A subject's `manifest.toml` — the complete, authoritative declaration.
- A subject's `CONTEXT.md` — its domain vocabulary.
- [`CONTEXT.md`](https://github.com/yanivholder2/cache-evolve/blob/main/CONTEXT.md) — the framework-level glossary.
