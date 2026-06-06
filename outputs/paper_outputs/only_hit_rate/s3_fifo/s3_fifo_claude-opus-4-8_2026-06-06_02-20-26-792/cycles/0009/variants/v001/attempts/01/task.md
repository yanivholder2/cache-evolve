# Task: variant-implementer

Variant: v001
Parent variant: cycle/0008/v001 (/home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-20-26-792/cycles/0008/variants/v001)
Hypothesis: /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-20-26-792/cycles/0009/hypotheses/H01.md
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-20-26-792/cycles/0009/variants/v001/attempts/01/overlay (overlay-by-path)

## Paths

- Parent overlay/:       /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-20-26-792/cycles/0008/variants/v001/attempts/01/overlay
- Parent fitness-vector: /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-20-26-792/cycles/0008/variants/v001/eval/fitness-vector.json
- Parent insight.md:     /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-20-26-792/cycles/0008/variants/v001/insight.md
- Subject src/:          /home/user/cache-evolve/inputs/subjects/s3_fifo/src
- Subject manifest:      /home/user/cache-evolve/inputs/subjects/s3_fifo/manifest.toml

## Authoring scope

full-autonomy

Gate-integrity files are pinned to the pristine crate and may not be
authored in any scope — overlay copies of them are ignored and rejected:
src/s3_fifo/s3_fifo_jni.rs, Cargo.toml, src/lib.rs, tests/, src/test_utils/

## Write

Place every changed file under `overlay/<path-within-subject>`; the overlay
is applied to a copy of the subject crate at build time (gate-integrity
files excluded).
