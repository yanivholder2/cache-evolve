# Task: variant-implementer

Variant: v001
Parent variant: cycle/0003/v003 (/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0003/variants/v003)
Hypothesis: /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0004/hypotheses/H01.md
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0004/variants/v001/attempts/01/overlay (overlay-by-path)

## Paths

- Parent overlay/:       /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0003/variants/v003/attempts/01/overlay
- Parent fitness-vector: /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0003/variants/v003/eval/fitness-vector.json
- Parent insight.md:     /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0003/variants/v003/insight.md
- Subject src/:          /home/user/cache-evolve/inputs/subjects/dash/src
- Subject manifest:      /home/user/cache-evolve/inputs/subjects/dash/manifest.toml

## Authoring scope

full-autonomy

Gate-integrity files are pinned to the pristine crate and may not be
authored in any scope — overlay copies of them are ignored and rejected:
src/dash/jni.rs, src/factory.rs, Cargo.toml, src/lib.rs, tests/, src/test_utils/, src/bin/

## Write

Place every changed file under `overlay/<path-within-subject>`; the overlay
is applied to a copy of the subject crate at build time (gate-integrity
files excluded).
