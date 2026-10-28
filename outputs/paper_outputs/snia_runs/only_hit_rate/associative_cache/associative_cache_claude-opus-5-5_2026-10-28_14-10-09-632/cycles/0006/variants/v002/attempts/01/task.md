# Task: variant-implementer

Variant: v002
Parent variant: cycle/0005/v003 (/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0005/variants/v003)
Hypothesis: /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0006/hypotheses/H02.md
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0006/variants/v002/attempts/01/overlay (overlay-by-path)

## Paths

- Parent overlay/:       /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0005/variants/v003/attempts/01/overlay
- Parent fitness-vector: /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0005/variants/v003/eval/fitness-vector.json
- Parent insight.md:     /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0005/variants/v003/insight.md
- Subject src/:          /home/user/cache-evolve/inputs/subjects/associative_cache/src
- Subject manifest:      /home/user/cache-evolve/inputs/subjects/associative_cache/manifest.toml

## Authoring scope

full-autonomy

Gate-integrity files are pinned to the pristine crate and may not be
authored in any scope — overlay copies of them are ignored and rejected:
src/associative_cache/associative_cache_jni.rs, src/factory.rs, Cargo.toml, src/lib.rs, tests/, src/test_utils/, src/bin/

## Write

Place every changed file under `overlay/<path-within-subject>`; the overlay
is applied to a copy of the subject crate at build time (gate-integrity
files excluded).
