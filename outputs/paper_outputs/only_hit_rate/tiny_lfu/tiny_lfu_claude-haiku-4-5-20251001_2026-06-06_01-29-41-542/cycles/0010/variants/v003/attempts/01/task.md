# Task: variant-implementer

Variant: v003
Parent variant: baseline (/home/yaniv/projects/cache-evolve/inputs/subjects/tiny_lfu/baseline)
Hypothesis: /home/yaniv/projects/cache-evolve/outputs/paper_outputs/tiny_lfu/tiny_lfu_claude-haiku-4-5-20251001_2026-06-06_01-29-41-542/cycles/0010/hypotheses/H03.md
Output dir: /home/yaniv/projects/cache-evolve/outputs/paper_outputs/tiny_lfu/tiny_lfu_claude-haiku-4-5-20251001_2026-06-06_01-29-41-542/cycles/0010/variants/v003/attempts/01/overlay (overlay-by-path)

## Paths

- Parent overlay/:       /home/yaniv/projects/cache-evolve/inputs/subjects/tiny_lfu/baseline/overlay
- Parent fitness-vector: /home/yaniv/projects/cache-evolve/inputs/subjects/tiny_lfu/baseline/eval/fitness-vector.json
- Parent insight.md:     /home/yaniv/projects/cache-evolve/inputs/subjects/tiny_lfu/baseline/insight.md
- Subject src/:          /home/yaniv/projects/cache-evolve/inputs/subjects/tiny_lfu/src
- Subject manifest:      /home/yaniv/projects/cache-evolve/inputs/subjects/tiny_lfu/manifest.toml

## Authoring scope

full-autonomy

Gate-integrity files are pinned to the pristine crate and may not be
authored in any scope — overlay copies of them are ignored and rejected:
src/tiny_lfu/tiny_lfu_jni.rs, Cargo.toml, src/lib.rs, tests/, src/test_utils/

## Write

Place every changed file under `overlay/<path-within-subject>`; the overlay
is applied to a copy of the subject crate at build time (gate-integrity
files excluded).
