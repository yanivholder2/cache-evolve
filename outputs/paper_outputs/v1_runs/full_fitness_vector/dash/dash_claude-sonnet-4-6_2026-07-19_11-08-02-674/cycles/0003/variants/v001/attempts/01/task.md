# Task: variant-implementer

Variant: v001
Parent variant: cycle/0002/v001 (C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0002\variants\v001)
Hypothesis: C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0003\hypotheses\H01.md
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0003\variants\v001\attempts\01\overlay (overlay-by-path)

## Paths

- Parent overlay/:       C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0002\variants\v001\attempts\01\overlay
- Parent fitness-vector: C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0002\variants\v001\eval\fitness-vector.json
- Parent insight.md:     C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0002\variants\v001\insight.md
- Subject src/:          C:\Users\yaniv\Projects\cache-evolve\inputs\subjects\dash\src
- Subject manifest:      C:\Users\yaniv\Projects\cache-evolve\inputs\subjects\dash\manifest.toml

## Authoring scope

full-autonomy

Gate-integrity files are pinned to the pristine crate and may not be
authored in any scope — overlay copies of them are ignored and rejected:
src\dash\jni.rs, src\factory.rs, Cargo.toml, src\lib.rs, tests/, src\test_utils/, src\bin/

## Write

Place every changed file under `overlay/<path-within-subject>`; the overlay
is applied to a copy of the subject crate at build time (gate-integrity
files excluded).
