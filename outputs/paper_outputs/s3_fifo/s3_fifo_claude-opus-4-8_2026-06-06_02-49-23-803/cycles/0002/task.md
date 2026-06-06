# Task: cycle-insight-writer

Cycle: 0002
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0002

## Winners

- v001
- v002
- v003

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0002/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0002/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0002/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0002/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0002/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0002/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants were single-knob tweaks to the baseline S3-FIFO,
each aimed at lifting hit-rate on workload-a by retaining more
reuse-proven entries. Only the capacity-split change (shrinking the
small admission queue) actually raised hit-rate; the two "admit/protect
more aggressively" changes both lowered it, and the cohort's lone real
winner was the bucket-resizing variant.

## Patterns

- Two distinct strategies were tried: **looser retention** (promote on
  first reuse; re-admit ghosts at non-zero freq) and **reallocation**
  (move capacity from small to main). Both retention-loosening moves
  hurt hit-rate; the reallocation move helped.
- The baseline's "strictness" is load-bearing: the two-touch promotion
  threshold and the freq=0 ghost re-admission both act as noise filters
  that keep short-lived, weakly-reused items from overstaying in main.
  Weakening either let one-touch items crowd out durable entries.
- Giving the main queue more *room* beats giving individual entries more
  *protection*: the small queue only needs to be wide enough to screen
  one-hit-wonders, and freed slots are better spent on main residency.
- Reported per-op-time movements appear across variants that add no
  hot-path work and don't change the scanned resident-set size — these
  read as measurement variance, not real effects.

## Open questions

- How far can the small/main split be pushed before the small queue is
  too narrow to screen one-hit-wonders effectively?
- Are admission-side and retention-side knobs genuinely orthogonal, or
  does a smaller small queue change the value of the promotion
  threshold?

## Continuation

Pursue capacity-allocation knobs (small/main split, ghost sizing) — that
is where hit-rate moved. Drop the "make main stickier" family
(first-reuse promotion, non-zero ghost re-admission): on this workload
the baseline's strict filtering is protective, not over-aggressive.
Next cycle should sweep the split ratio further and consider pairing it
with the *unchanged* strict thresholds rather than loosening them.
Treat per-op-time deltas as noise unless a change touches the hot path
or resident-set size.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
