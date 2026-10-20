# Task: variant-implementer

Variant: v001
Parent variant: cycle/0003/v002 (C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0003\variants\v002)
Hypothesis: C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0004\hypotheses\H01.md
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0004\variants\v001\attempts\01\overlay (overlay-by-path)

## Paths

- Parent overlay/:       C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0003\variants\v002\attempts\01\overlay
- Parent fitness-vector: C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0003\variants\v002\eval\fitness-vector.json
- Parent insight.md:     C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0003\variants\v002\insight.md
- Subject src/:          C:\Users\yaniv\Projects\cache-evolve\inputs\subjects\s3_fifo\src
- Subject manifest:      C:\Users\yaniv\Projects\cache-evolve\inputs\subjects\s3_fifo\manifest.toml

## Authoring scope

full-autonomy

Gate-integrity files are pinned to the pristine crate and may not be
authored in any scope — overlay copies of them are ignored and rejected:
src\s3_fifo\s3_fifo_jni.rs, Cargo.toml, src\lib.rs, tests/, src\test_utils/, src\bin/

## Write

Place every changed file under `overlay/<path-within-subject>`; the overlay
is applied to a copy of the subject crate at build time (gate-integrity
files excluded).
