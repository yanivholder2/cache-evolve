# Task: variant-implementer

Variant: v003
Parent variant: baseline (/home/user/cache-evolve/inputs/subjects/lru_cache/baseline)
Hypothesis: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0003/hypotheses/H03.md
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0003/variants/v003/attempts/02/overlay (overlay-by-path)

## Paths

- Parent overlay/:       /home/user/cache-evolve/inputs/subjects/lru_cache/baseline/overlay
- Parent fitness-vector: /home/user/cache-evolve/inputs/subjects/lru_cache/baseline/eval/fitness-vector.json
- Parent insight.md:     /home/user/cache-evolve/inputs/subjects/lru_cache/baseline/insight.md
- Subject src/:          /home/user/cache-evolve/inputs/subjects/lru_cache/src
- Subject manifest:      /home/user/cache-evolve/inputs/subjects/lru_cache/manifest.toml

## Authoring scope

full-autonomy

Gate-integrity files are pinned to the pristine crate and may not be
authored in any scope — overlay copies of them are ignored and rejected:
src/lru_cache/lru_cache_jni.rs, Cargo.toml, src/lib.rs, tests/, src/test_utils/, src/bin/

## Failing gate
The previous attempt's overlay was rejected. Diagnose and fix it:
- Previous overlay (the parent you are fixing): /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0003/variants/v003/attempts/01/overlay

Latest rejection entry:

## 2026-10-20T14:15:26Z — variant=cycle/0003/v003 hypothesis=H03 attempt=1

**Stage**: implement

**Problem**: adapter produced no overlay files under /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0003/variants/v003/attempts/01/overlay

Write at least one changed subject file under overlay/; an empty overlay is indistinguishable from the baseline.

## Write

Place every changed file under `overlay/<path-within-subject>`; the overlay
is applied to a copy of the subject crate at build time (gate-integrity
files excluded).
