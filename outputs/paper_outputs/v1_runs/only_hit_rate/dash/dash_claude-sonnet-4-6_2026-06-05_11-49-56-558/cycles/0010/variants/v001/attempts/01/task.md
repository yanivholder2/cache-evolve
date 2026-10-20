# Task: variant-implementer

Variant: v001
Parent variant: cycle/0009/v003 (/home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cycles/0009/variants/v003)
Hypothesis: /home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cycles/0010/hypotheses/H01.md
Output dir: /home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cycles/0010/variants/v001/attempts/01/overlay (overlay-by-path)

## Paths

- Parent overlay/:       /home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cycles/0009/variants/v003/attempts/01/overlay
- Parent fitness-vector: /home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cycles/0009/variants/v003/eval/fitness-vector.json
- Parent insight.md:     /home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cycles/0009/variants/v003/insight.md
- Subject src/:          /home/yaniv/projects/cache-evolve/inputs/subjects/dash/src
- Subject manifest:      /home/yaniv/projects/cache-evolve/inputs/subjects/dash/manifest.toml

## Authoring scope

extension-points

Gate-integrity files are pinned to the pristine crate and may not be
authored in any scope — overlay copies of them are ignored and rejected:
src/dash/jni.rs, src/factory.rs, Cargo.toml, src/lib.rs, tests/, src/test_utils/

## Write

Place every changed file under `overlay/<path-within-subject>`; the overlay
is applied to a copy of the subject crate at build time (gate-integrity
files excluded).
