# Task: variant-implementer

Variant: v002
Parent variant: cycle/0008/v001 (/home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0008/variants/v001)
Hypothesis: /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0010/hypotheses/H02.md
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0010/variants/v002/attempts/02/overlay (overlay-by-path)

## Paths

- Parent overlay/:       /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0008/variants/v001/attempts/01/overlay
- Parent fitness-vector: /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0008/variants/v001/eval/fitness-vector.json
- Parent insight.md:     /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0008/variants/v001/insight.md
- Subject src/:          /home/user/cache-evolve/inputs/subjects/dash/src
- Subject manifest:      /home/user/cache-evolve/inputs/subjects/dash/manifest.toml

## Authoring scope

full-autonomy

Gate-integrity files are pinned to the pristine crate and may not be
authored in any scope — overlay copies of them are ignored and rejected:
src/dash/jni.rs, src/factory.rs, Cargo.toml, src/lib.rs, tests/, src/test_utils/, src/bin/

## Failing gate
The previous attempt's overlay was rejected. Diagnose and fix it:
- Previous overlay (the parent you are fixing): /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0010/variants/v002/attempts/01/overlay

Latest rejection entry:

## 2026-10-20T12:14:09Z — variant=cycle/0010/v002 hypothesis=H02 attempt=1

**Stage**: implement

**Problem**: adapter produced no overlay files under /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0010/variants/v002/attempts/01/overlay

Write at least one changed subject file under overlay/; an empty overlay is indistinguishable from the baseline.

## Write

Place every changed file under `overlay/<path-within-subject>`; the overlay
is applied to a copy of the subject crate at build time (gate-integrity
files excluded).
