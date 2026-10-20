# Task: cycle-insight-writer

Cycle: 0002
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0002

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0002/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0002/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0002/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0002/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0002/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0002/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

The cohort tried three changes to how items move between the probing, stash, and normal tiers: where a stash-promoted item lands in its target bucket, whether a probing hit re-inserts the item, and whether foreign evictions skip the retry in their true target. Only v002, the change to the landing slot for stash hits, won; the other two regressed on hit-rate and were not winners.

## Patterns

- Landing position matters more than movement. Moving a re-referenced stash item to the MRU end of its target bucket was the only change that helped, while the two changes that added movement or removed a retry both hurt hit-rate.
- Keeping residency in the probing tier appears valuable. Removing and re-inserting probing hits pushed proven-hot keys back through target competition and lost ground.
- Shortening the demotion path alone does not pay off. Skipping the target retry for foreign evictions lowered the one measured dimension, and the predicted per-op-time gain was never recorded.
- The winner's evidence is thin. Its only measured dimension is hit-rate, on a single workload, so the win is a hit-rate result, not a demonstrated overall improvement.
- Rationale is inconsistently documented around the winning change. The Front/Back default is described differently in different places, which makes the intent of v002 harder for later cycles to verify.

## Open questions

- Does the MRU landing for stash hits hold on workloads other than the LIRS trace used here?
- Do per-op-time and metadata-size move at all under any of these changes? No variant measured them.
- Would the removed target retry pay for itself if the shorter eviction path is paired with better stash capacity handling?

## Continuation

Next cycle should build on v002: vary the stash-hit landing slot further and measure it across more workloads, plus per-op-time and metadata-size. Drop the probing-hit remove-and-reinsert idea and the direct-to-stash eviction change unless a follow-up explicitly measures the costs they were meant to offset.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
