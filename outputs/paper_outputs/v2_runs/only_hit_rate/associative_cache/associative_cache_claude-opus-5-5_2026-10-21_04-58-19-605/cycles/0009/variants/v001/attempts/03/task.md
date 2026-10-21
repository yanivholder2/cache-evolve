# Task: variant-implementer

Variant: v001
Parent variant: cycle/0007/v001 (/home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0007/variants/v001)
Hypothesis: /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0009/hypotheses/H01.md
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0009/variants/v001/attempts/03/overlay (overlay-by-path)

## Paths

- Parent overlay/:       /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0007/variants/v001/attempts/01/overlay
- Parent fitness-vector: /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0007/variants/v001/eval/fitness-vector.json
- Parent insight.md:     /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0007/variants/v001/insight.md
- Subject src/:          /home/user/cache-evolve/inputs/subjects/associative_cache/src
- Subject manifest:      /home/user/cache-evolve/inputs/subjects/associative_cache/manifest.toml

## Authoring scope

full-autonomy

Gate-integrity files are pinned to the pristine crate and may not be
authored in any scope — overlay copies of them are ignored and rejected:
src/associative_cache/associative_cache_jni.rs, src/factory.rs, Cargo.toml, src/lib.rs, tests/, src/test_utils/, src/bin/

## Failing gate
The previous attempt's overlay was rejected. Diagnose and fix it:
- Previous overlay (the parent you are fixing): /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0009/variants/v001/attempts/02/overlay

Latest rejection entry:

## 2026-10-21T09:34:46Z — variant=cycle/0009/v001 hypothesis=H01 attempt=2

**Stage**: implement

**Problem**: adapter produced no overlay files under /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0009/variants/v001/attempts/02/overlay

Write at least one changed subject file under overlay/; an empty overlay is indistinguishable from the baseline.

## Write

Place every changed file under `overlay/<path-within-subject>`; the overlay
is applied to a copy of the subject crate at build time (gate-integrity
files excluded).
