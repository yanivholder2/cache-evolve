## Recurring wins

- Making the existing sketch cheaper and more accurate (single hash, small counters, conservative update) paid off in speed and footprint with a small hit-rate gain. Only one cycle of evidence so far.
- Removing repeated hashing on the hot path gave the clearest per-op gain, especially in the put tail.

## Recurring failures

- Adding mechanism to the admission path (sketch aging, small admission window) gave no hit-rate gain on workload-a. It cost per-op time and struct metadata.
- Hypotheses premised on stale frequencies or newcomers needing a trial period did not hold on this trace. Baseline never-decaying counters and strict-greater admission fit it well.
- Extra fields or bookkeeping on the get/hit path raised latency with no matching benefit.

## Open questions

- Which of v003's parts (single hash, u8 counters, conservative update) account for the gains?
- Do u8 saturation and conservative update hold on traces with larger frequency gaps or drift?
- Would aging help with a much longer period or on a trace with real drift? Would a much larger window help? Retry these only in a materially different form (conservative aging, doorkeeper) tied to a specific trace property.
- Can per-op cost and footprint go lower still (cheaper hashing, tighter sketch layout, less hit-path work)?

## Per-policy notes

- TinyLFU (baseline, v003 as current parent): per-op cost is dominated by sketch hashing. The metadata-size headline tracks Node size, not sketch heap, so sketch savings are invisible there. Judge footprint gains separately.
- Prefer hypotheses whose gains show in the headline dimensions.
