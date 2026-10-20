# Task: variant-implementer

Variant: v002
Parent variant: cycle/0002/v002 (/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0002/variants/v002)
Hypothesis: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0006/hypotheses/H02.md
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0006/variants/v002/attempts/01/overlay (overlay-by-path)

## Paths

- Parent overlay/:       /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0002/variants/v002/attempts/01/overlay
- Parent fitness-vector: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0002/variants/v002/eval/fitness-vector.json
- Parent insight.md:     /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0002/variants/v002/insight.md
- Subject src/:          /home/user/cache-evolve/inputs/subjects/tiny_lfu/src
- Subject manifest:      /home/user/cache-evolve/inputs/subjects/tiny_lfu/manifest.toml

## Authoring scope

full-autonomy

Gate-integrity files are pinned to the pristine crate and may not be
authored in any scope — overlay copies of them are ignored and rejected:
src/tiny_lfu/tiny_lfu_jni.rs, Cargo.toml, src/lib.rs, tests/, src/test_utils/, src/bin/

## Write

Place every changed file under `overlay/<path-within-subject>`; the overlay
is applied to a copy of the subject crate at build time (gate-integrity
files excluded).
