# Task: variant-implementer

Variant: v002
Parent variant: baseline (C:\Users\yaniv\Projects\cache-evolve\inputs\subjects\associative_cache\baseline)
Hypothesis: C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_13-12-55-552\cycles\0001\hypotheses\H02.md
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_13-12-55-552\cycles\0001\variants\v002\attempts\01\overlay (overlay-by-path)

## Paths

- Parent overlay/:       C:\Users\yaniv\Projects\cache-evolve\inputs\subjects\associative_cache\baseline\overlay
- Parent fitness-vector: C:\Users\yaniv\Projects\cache-evolve\inputs\subjects\associative_cache\baseline\eval\fitness-vector.json
- Parent insight.md:     C:\Users\yaniv\Projects\cache-evolve\inputs\subjects\associative_cache\baseline\insight.md
- Subject src/:          C:\Users\yaniv\Projects\cache-evolve\inputs\subjects\associative_cache\src
- Subject manifest:      C:\Users\yaniv\Projects\cache-evolve\inputs\subjects\associative_cache\manifest.toml

## Authoring scope

full-autonomy

Gate-integrity files are pinned to the pristine crate and may not be
authored in any scope — overlay copies of them are ignored and rejected:
src\associative_cache\associative_cache_jni.rs, src\factory.rs, Cargo.toml, src\lib.rs, tests/, src\test_utils/, src\bin/

## Write

Place every changed file under `overlay/<path-within-subject>`; the overlay
is applied to a copy of the subject crate at build time (gate-integrity
files excluded).
