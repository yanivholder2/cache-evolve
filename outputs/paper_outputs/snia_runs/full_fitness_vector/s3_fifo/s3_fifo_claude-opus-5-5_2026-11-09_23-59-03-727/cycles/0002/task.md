# Task: cycle-insight-writer

Cycle: 0002
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0002

## Winners

- v003
- v002
- v001

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0002/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0002/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0002/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0002/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0002/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0002/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants were single-axis changes to how S3-FIFO splits capacity and history: a fixed 10% small queue (v001), ghosting main-queue victims (v002), and an ARC-style adaptive small/main split (v003). None of them clearly beat baseline on hit rate. v001 and v002 came out roughly neutral, trading gains on some trace/size points for losses on others, and v003 regressed.

## Patterns

- Gains and losses depend on the trace and the cache size. hm_0 tends to reward more main-queue residency or main-victim history. web_0 at the large size keeps losing whenever the probation window or the ghost history meant for small-queue demotees gets squeezed. Each change helped one corner of the trace×size grid and hurt the opposite corner.
- Ghost capacity is a shared, zero-sum resource. v002 and v003 both kept the ghost bound fixed while changing what competes for it or how big main is, and both showed the mismatch hurting. The ghost bound looks like a hidden coupled variable that cycle 1 never isolated.
- Every variant showed a large per-op-time improvement that its diff can't explain, including v003, which only adds work. This is a benchmark-environment shift, not a real effect. Per-op-time comparisons from this cycle are unreliable.
- Metadata per entry stayed flat across the cohort. Only v003 added a small fixed per-cache cost.

## Open questions

- Is the per-op-time drift caused by the baseline being measured in a different environment? Until a same-machine baseline is re-run, no variant should get timing credit.
- Would a ghost sized or split independently of `main_size`, for example with separate regions for small demotees and main victims, let v002's hm_0 gain survive without the web_0-large regression?
- Can adaptive sizing work if the two signals are balanced (normalized by queue size, or shrinking once per evicted main entry rather than per reinsertion), the ghost tracks the live main size, and the step is smaller at large capacity?

## Continuation

Drop fixed-ratio sweeps. A fixed ratio only moves the trade-off between hm_0 and web_0-large around without improving the summary. Next, try making ghost capacity its own variable, either decoupled from or split by source, and possibly a rebalanced adaptive split with a ghost that tracks main's live size. Re-baseline per-op-time in the same environment before trusting any timing result.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
