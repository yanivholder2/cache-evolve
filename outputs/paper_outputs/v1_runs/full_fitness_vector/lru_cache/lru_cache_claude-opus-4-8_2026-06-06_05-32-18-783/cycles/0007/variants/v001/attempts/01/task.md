# Task: variant-implementer

Variant: v001
Parent variant: cycle/0006/v003 (/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0006/variants/v003)
Hypothesis: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0007/hypotheses/H01.md
Output dir: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0007/variants/v001/attempts/01/overlay (overlay-by-path)

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

## Write

Place every changed file under `overlay/<path-within-subject>`; the overlay
is applied to a copy of the subject crate at build time (gate-integrity
files excluded).
