# Task: variant-implementer

Variant: v003
Parent variant: cycle/0008/v001 (/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0008/variants/v001)
Hypothesis: /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0009/hypotheses/H03.md
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0009/variants/v003/attempts/01/overlay (overlay-by-path)

## Paths

- Parent overlay/:       /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0008/variants/v001/attempts/01/overlay
- Parent fitness-vector: /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0008/variants/v001/eval/fitness-vector.json
- Parent insight.md:     /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0008/variants/v001/insight.md
- Subject src/:          /home/user/cache-evolve/inputs/subjects/tiny_lfu/src
- Subject manifest:      /home/user/cache-evolve/inputs/subjects/tiny_lfu/manifest.toml

## Authoring scope

full-autonomy

Gate-integrity files are pinned to the pristine crate and may not be
authored in any scope — overlay copies of them are ignored and rejected:
src/tiny_lfu/tiny_lfu_jni.rs, Cargo.toml, src/lib.rs, tests/, src/test_utils/

## Write

Place every changed file under `overlay/<path-within-subject>`; the overlay
is applied to a copy of the subject crate at build time (gate-integrity
files excluded).
