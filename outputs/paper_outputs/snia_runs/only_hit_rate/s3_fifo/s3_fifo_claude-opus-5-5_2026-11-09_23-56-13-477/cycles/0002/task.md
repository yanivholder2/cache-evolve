# Task: cycle-insight-writer

Cycle: 0002
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0002

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0002/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0002/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0002/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0002/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0002/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0002/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Three single-variable changes were tried against the baseline: a lower graduation threshold (v001), an O(1) hash-index/slab refactor with the policy left untouched (v002), and a 10/90 small/main split (v003). v001 won, but only barely. Neither policy change moved aggregate hit-rate much: each gained on hm_0 and lost on web_0.

## Patterns

- Both policy knobs (graduation threshold and small-queue size) split the two traces the same way. hm_0 rewards letting keys into main more easily or keeping more main capacity. web_0 penalises it, either through main pollution from keys accessed exactly twice or through medium-reuse keys getting ghosted early. A trace-agnostic constant change looks unlikely to win clearly on both.
- The two policy knobs interact. v003 points to the strict `freq > 1` threshold as what limits a smaller small queue, and v001 points to unfiltered admission as what limits a looser threshold. Each fix seems to need the other's lever.
- What happens to an entry once it is in main doesn't help. Resetting a graduate's freq didn't offset admission pollution, so the cost is paid when a key is admitted to main.
- The refactor kept hit-rate identical to the baseline, so the policy is unchanged. But only hit-rate was measured for this cohort, so the per-op-time and metadata-size effects are still unknown.

## Open questions

- Does v002 actually reduce per-op-time, and does the extra index, slab and stamp state make metadata-size worse?
- Does combining a small queue between 10% and 25% with a threshold of 1 keep hm_0's gain without web_0's loss?
- Can an adaptive signal tell the two traces apart online, such as how much spare room main has or how often keys are reused in the small queue?

## Continuation

Check that per-op-time and metadata-size are being evaluated, and re-measure v002. If it is confirmed faster, use it as the parent for later policy work. Stop trying single-constant tweaks to the threshold or the split. Instead, change the threshold and the small-queue size together, or make graduation into main adaptive (conditioned on main having room or on the observed reuse rate), so that web_0 doesn't regress.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
