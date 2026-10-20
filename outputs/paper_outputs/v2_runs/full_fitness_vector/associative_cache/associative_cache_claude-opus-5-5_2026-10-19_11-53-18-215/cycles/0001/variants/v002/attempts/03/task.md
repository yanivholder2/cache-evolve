# Task: variant-implementer

Variant: v002
Parent variant: baseline (/home/user/cache-evolve/inputs/subjects/associative_cache/baseline)
Hypothesis: /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0001/hypotheses/H02.md
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0001/variants/v002/attempts/03/overlay (overlay-by-path)

## Paths

- Parent overlay/:       /home/user/cache-evolve/inputs/subjects/associative_cache/baseline/overlay
- Parent fitness-vector: /home/user/cache-evolve/inputs/subjects/associative_cache/baseline/eval/fitness-vector.json
- Parent insight.md:     /home/user/cache-evolve/inputs/subjects/associative_cache/baseline/insight.md
- Subject src/:          /home/user/cache-evolve/inputs/subjects/associative_cache/src
- Subject manifest:      /home/user/cache-evolve/inputs/subjects/associative_cache/manifest.toml

## Authoring scope

full-autonomy

Gate-integrity files are pinned to the pristine crate and may not be
authored in any scope — overlay copies of them are ignored and rejected:
src/associative_cache/associative_cache_jni.rs, src/factory.rs, Cargo.toml, src/lib.rs, tests/, src/test_utils/, src/bin/

## Failing gate
The previous attempt's overlay was rejected. Diagnose and fix it:
- Previous overlay (the parent you are fixing): /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0001/variants/v002/attempts/02/overlay

Latest rejection entry:

## 2026-10-19T11:57:40Z — variant=cycle/0001/v002 hypothesis=H02 attempt=2

**Stage**: implement

**Problem**: adapter produced no overlay files under /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0001/variants/v002/attempts/02/overlay

Write at least one changed subject file under overlay/; an empty overlay is indistinguishable from the baseline.

## Write

Place every changed file under `overlay/<path-within-subject>`; the overlay
is applied to a copy of the subject crate at build time (gate-integrity
files excluded).
