## Recurring wins

- Being strict about which keys get into main is the one lever that has helped in both cycles. Every gain came from tightening or keeping the baseline two-reuse graduation rule. Loosening it never gained in steady state.
- A freq reset on graduation does no harm, and the winning lineage (v001 → v001) carries it. It doesn't offset pollution at admission, though.
- The O(1) hash-index/slab refactor kept hit-rate identical to the baseline. That makes it a safe base for policy work, but its cost benefit is still unconfirmed.

## Recurring failures

- Changing one policy constant (graduation threshold or small/main split) doesn't win cleanly. Its effect flips sign across traces (hm_0 vs web_0) and across cache sizes, and the movement is close to noise.
- Static split tuning has failed in both cycles, at 10/90 and at 15% combined with the looser threshold. Queue sizes matter less than which keys get into main.
- The one-reuse threshold pollutes main with keys accessed exactly twice. Its apparent gain on hm_0 is a cold-start or small-cache effect that disappears at larger sizes and in steady state.
- Main occupancy doesn't work as an adaptive signal: main fills up and stays full, so any gate based on it falls back to the baseline rule.
- An unweighted ARC-style ghost-hit split probably drifts toward a larger small queue because the two ghosts differ in size. It also adds a second linear ghost scan on every miss.
- Per-op-time and metadata-size have gone unmeasured for two cycles running. Every variant has followed the hit-rate policy line, so the requested re-measurement keeps getting skipped.

## Open questions

- How much per-op-time does the O(1) refactor actually save, and what does its index, slab and stamp state cost in metadata-size? This is now the highest-priority gap.
- Can a signal that keeps changing in steady state (ghost hit rate, small-queue reuse rate) usefully switch the threshold between 1 and 2? Or is the baseline rule simply right for these traces?
- Where does an adaptive `small_size` settle on each trace? This has never been logged, and a size-weighted ARC step would answer it.
- How much per-op-time does the extra linear `main_ghost` scan cost, and would an O(1) ghost structure remove that cost?

## Per-policy notes

- S3-FIFO baseline: the two-reuse (`freq > 1`) graduation rule into main appears near-optimal for hm_0/web_0 under steady-state pressure. Proposed changes have mostly ended up back at it.
- Graduation threshold: a threshold of 1 helps only hm_0 at the small cache size and hurts web_0 and hm_0 at the large size. The threshold and small-queue size interact, so if they're ever tuned, they should be tuned together.
- Small/main split: static and unweighted adaptive splits both give effects at noise level whose sign flips. Don't use more hypotheses on this unless the ARC step is size-weighted and logs where `small_size` settles.
- Adaptive gates: gate on a signal that keeps changing in steady state, never on occupancy.
- Implementation line: the O(1) index/slab refactor doesn't change policy behaviour. The policy work should be moved onto it once per-op-time and metadata-size have been measured.
