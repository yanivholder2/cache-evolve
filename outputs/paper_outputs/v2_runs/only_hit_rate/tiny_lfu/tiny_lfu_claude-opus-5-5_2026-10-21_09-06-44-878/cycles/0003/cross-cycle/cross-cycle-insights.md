## Recurring wins

- Changes that make frequency estimates more accurate pay off. Fast hashing with a flat sketch (cycle 1) helped a little, probably through different collisions. A wider count-min sketch (cycle 3) gave the largest policy gain so far. Lower collision noise is the strongest lever found to date.
- On workload-a, long-run frequency is worth keeping. Every result points the same way: lifetime counts beat freshness.
- Small, single-lever changes on a stable lineage (v003 → c2 v001 → c3 v003) produce readable results. Each Winner was a modest change that left the admission contest's semantics alone.

## Recurring failures

- Sketch aging/decay has lost twice (cycle 1 with conservative update, cycle 2 alone with a long period). It throws away frequency history the trace rewards and creates ties that newcomers lose. Both tests ran on the noisy narrow sketch, so the results are confounded.
- Loosening the contest has failed. Admitting on ties (cycle 3) swapped one cold key for another. Strict `>` with keep-victim-on-tie is now validated.
- Victim sampling has stalled. It gave a marginal win in cycle 2, then widening it lost in cycle 3: with estimates that never age, the lowest estimate marks the youngest resident rather than the coldest.
- Restructuring the window and segments has failed. A fixed 1% window changed nothing (cycle 1). An adaptive hill-climber grew the window toward LRU-like behavior, which this LIRS-style trace punishes (cycle 2).
- Per-op-time and metadata-size have gone unmeasured in all three cycles. No latency claim (fast hashing) or memory cost (wider sketch, extra estimates on miss-puts) has been checked.

## Open questions

- With collision noise reduced, does very-long-period aging finally help, or does lifetime frequency still win on workload-a?
- Would a doorkeeper, or deduplicating the get-miss-then-put double increment, help estimates tell one-hit wonders apart?
- How far does sketch width/depth scale before hit-rate plateaus, and what does that cost in memory and latency?
- What do the accumulated changes (fast hash, sampled victims, wider sketch) actually cost in per-op-time and metadata-size?
- Do the conclusions hold beyond workload-a? Every finding so far comes from one LIRS-style trace.

## Per-policy notes

- TinyLFU admission (sketch + strict-`>` contest): the core is sound. Gains come from estimate quality, not from changing the contest rule. The bottlenecks are collision overcount and the get/put double count.
- W-TinyLFU window/SLRU layout: inert at a 1% window and harmful when adapted naively. Keep the window small and fixed unless a damped, warmed-up climber is tested.
- Count-min sketch: making it wider is the current winning direction. Conservative update has only been tested together with aggressive aging, so it is unresolved on its own.
- Victim selection (LRU-head sampling): a small benefit at a narrow sample, a loss at a wider one. Treat it as settled.
