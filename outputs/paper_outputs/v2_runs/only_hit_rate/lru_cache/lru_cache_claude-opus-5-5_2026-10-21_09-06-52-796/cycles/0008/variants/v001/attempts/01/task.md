# Task: variant-implementer

Variant: v001
Parent variant: cycle/0007/v002 (/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0007/variants/v002)
Hypothesis: /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0008/hypotheses/H01.md
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0008/variants/v001/attempts/01/overlay (overlay-by-path)

## Paths

- Parent overlay/:       /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0007/variants/v002/attempts/01/overlay
- Parent fitness-vector: /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0007/variants/v002/eval/fitness-vector.json
- Parent insight.md:     /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0007/variants/v002/insight.md
- Subject src/:          /home/user/cache-evolve/inputs/subjects/lru_cache/src
- Subject manifest:      /home/user/cache-evolve/inputs/subjects/lru_cache/manifest.toml

## Authoring scope

full-autonomy

Gate-integrity files are pinned to the pristine crate and may not be
authored in any scope — overlay copies of them are ignored and rejected:
src/lru_cache/lru_cache_jni.rs, Cargo.toml, src/lib.rs, tests/, src/test_utils/, src/bin/

## Write

Place every changed file under `overlay/<path-within-subject>`; the overlay
is applied to a copy of the subject crate at build time (gate-integrity
files excluded).
