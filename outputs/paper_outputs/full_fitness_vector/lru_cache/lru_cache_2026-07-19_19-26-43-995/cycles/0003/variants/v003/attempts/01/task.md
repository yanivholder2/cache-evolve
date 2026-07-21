# Task: variant-implementer

Variant: v003
Parent variant: baseline (C:\Users\yaniv\Projects\cache-evolve\inputs\subjects\lru_cache\baseline)
Hypothesis: C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0003\hypotheses\H03.md
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0003\variants\v003\attempts\01\overlay (overlay-by-path)

## Paths

- Parent overlay/:       C:\Users\yaniv\Projects\cache-evolve\inputs\subjects\lru_cache\baseline\overlay
- Parent fitness-vector: C:\Users\yaniv\Projects\cache-evolve\inputs\subjects\lru_cache\baseline\eval\fitness-vector.json
- Parent insight.md:     C:\Users\yaniv\Projects\cache-evolve\inputs\subjects\lru_cache\baseline\insight.md
- Subject src/:          C:\Users\yaniv\Projects\cache-evolve\inputs\subjects\lru_cache\src
- Subject manifest:      C:\Users\yaniv\Projects\cache-evolve\inputs\subjects\lru_cache\manifest.toml

## Authoring scope

full-autonomy

Gate-integrity files are pinned to the pristine crate and may not be
authored in any scope — overlay copies of them are ignored and rejected:
src\lru_cache\lru_cache_jni.rs, Cargo.toml, src\lib.rs, tests/, src\test_utils/, src\bin/

## Write

Place every changed file under `overlay/<path-within-subject>`; the overlay
is applied to a copy of the subject crate at build time (gate-integrity
files excluded).
