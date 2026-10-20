# Task: variant-implementer

Variant: v003
Parent variant: cycle/0008/v002 (C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0008\variants\v002)
Hypothesis: C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0009\hypotheses\H03.md
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0009\variants\v003\attempts\01\overlay (overlay-by-path)

## Paths

- Parent overlay/:       C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0008\variants\v002\attempts\01\overlay
- Parent fitness-vector: C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0008\variants\v002\eval\fitness-vector.json
- Parent insight.md:     C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0008\variants\v002\insight.md
- Subject src/:          C:\Users\yaniv\Projects\cache-evolve\inputs\subjects\tiny_lfu\src
- Subject manifest:      C:\Users\yaniv\Projects\cache-evolve\inputs\subjects\tiny_lfu\manifest.toml

## Authoring scope

full-autonomy

Gate-integrity files are pinned to the pristine crate and may not be
authored in any scope — overlay copies of them are ignored and rejected:
src\tiny_lfu\tiny_lfu_jni.rs, Cargo.toml, src\lib.rs, tests/, src\test_utils/, src\bin/

## Write

Place every changed file under `overlay/<path-within-subject>`; the overlay
is applied to a copy of the subject crate at build time (gate-integrity
files excluded).
