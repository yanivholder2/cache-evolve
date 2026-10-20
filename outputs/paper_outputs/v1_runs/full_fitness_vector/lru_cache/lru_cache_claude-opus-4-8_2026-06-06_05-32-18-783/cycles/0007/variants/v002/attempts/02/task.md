# Task: variant-implementer

Variant: v002
Parent variant: cycle/0006/v003 (/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0006/variants/v003)
Hypothesis: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0007/hypotheses/H02.md
Output dir: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0007/variants/v002/attempts/02/overlay (overlay-by-path)

## Paths

- Parent overlay/:       /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0006/variants/v003/attempts/01/overlay
- Parent fitness-vector: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0006/variants/v003/eval/fitness-vector.json
- Parent insight.md:     /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0006/variants/v003/insight.md
- Subject src/:          /home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/src
- Subject manifest:      /home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/manifest.toml

## Authoring scope

full-autonomy

Gate-integrity files are pinned to the pristine crate and may not be
authored in any scope — overlay copies of them are ignored and rejected:
src/lru_cache/lru_cache_jni.rs, Cargo.toml, src/lib.rs, tests/, src/test_utils/, src/bin/

## Failing gate
The previous attempt's overlay was rejected. Diagnose and fix it:
- Previous overlay (the parent you are fixing): /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0007/variants/v002/attempts/01/overlay

Latest rejection entry:

## 2026-06-06T05:14:57Z — variant=cycle/0007/v002 hypothesis=H02 attempt=1

**Gate**: build gate
**Error**:
```
    Updating crates.io index
     Locking 114 packages to latest compatible versions
      Adding jni v0.20.0 (available: v0.22.4)
      Adding rand v0.8.6 (available: v0.10.1)
      Adding simplelog v0.11.2 (available: v0.12.2)
      Adding termcolor v1.1.3 (available: v1.4.1)
error: can't find lib `lru_cache` at path `/tmp/variant-workspace-5odnm9e4/src/lib.rs`
 --> /tmp/variant-workspace-5odnm9e4/Cargo.toml
error: can't find bin `lru_cache-evolve-metadata-size` at path `/tmp/variant-workspace-5odnm9e4/src/bin/evolve_metadata_size.rs`
 --> /tmp/variant-workspace-5odnm9e4/Cargo.toml
error: can't find bin `lru_cache-evolve-per-op-time` at path `/tmp/variant-workspace-5odnm9e4/src/bin/evolve_per_op_time.rs`
 --> /tmp/variant-workspace-5odnm9e4/Cargo.toml
error: can't find integration-test `contract_gate` at path `/tmp/variant-workspace-5odnm9e4/tests/contract_gate.rs`
 --> /tmp/variant-workspace-5odnm9e4/Cargo.toml
error: can't find integration-test `invariant_gate` at path `/tmp/variant-workspace-5odnm9e4/tests/invariant_gate.rs`
 --> /tmp/variant-workspace-5odnm9e4/Cargo.toml
error: can't find integration-test `lru_cache_integration` at path `/tmp/variant-workspace-5odnm9e4/tests/lru_cache_integration.rs`
 --> /tmp/variant-workspace-5odnm9e4/Cargo.toml
error: can't find integration-test `universal_invariant_gate` at path `/tmp/variant-workspace-5odnm9e4/tests/universal_invariant_gate.rs`
 --> /tmp/variant-workspace-5odnm9e4/Cargo.toml
error: could not compile due to 7 previous target resolution errors

```
---

## Write

Place every changed file under `overlay/<path-within-subject>`; the overlay
is applied to a copy of the subject crate at build time (gate-integrity
files excluded).
