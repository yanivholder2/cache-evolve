# Task: variant-implementer

Variant: v001
Parent variant: baseline (/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/baseline)
Hypothesis: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-22-14-433/cycles/0001/hypotheses/H01.md
Output dir: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-22-14-433/cycles/0001/variants/v001/attempts/02/overlay (overlay-by-path)

## Paths

- Parent overlay/:       /home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/baseline/overlay
- Parent fitness-vector: /home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/baseline/eval/fitness-vector.json
- Parent insight.md:     /home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/baseline/insight.md
- Subject src/:          /home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/src
- Subject manifest:      /home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/manifest.toml

## Authoring scope

full-autonomy

## Failing gate
The previous attempt's overlay was rejected. Diagnose and fix it:
- Previous overlay (the parent you are fixing): /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-22-14-433/cycles/0001/variants/v001/attempts/01/overlay

Latest rejection entry:

## 2026-06-04T20:30:34Z — variant=cycle/0001/v001 hypothesis=H01 attempt=1

**Stage**: implement

**Problem**: adapter produced no overlay files under /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-22-14-433/cycles/0001/variants/v001/attempts/01/overlay

Write at least one changed subject file under overlay/; an empty overlay is indistinguishable from the baseline.

## Write

Place every changed file under `overlay/<path-within-subject>`; the overlay
is applied verbatim to a copy of the subject crate at build time.
