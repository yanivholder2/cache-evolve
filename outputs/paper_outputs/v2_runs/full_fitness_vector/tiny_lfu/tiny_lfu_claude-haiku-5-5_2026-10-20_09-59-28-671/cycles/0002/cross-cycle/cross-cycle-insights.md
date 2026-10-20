## Recurring wins

- Constant-factor work that leaves eviction order untouched is the most reliable source of gains. Both the newest-key relink guard (cycle 1) and the single-hash reuse on the put path (cycle 2) cut per-op time without a meaningful hit-rate change.
- Metadata trimming can be done independently of policy. Packing the protected state into existing node fields (cycle 2) shrank bookkeeping without moving hit rate, which makes it a safe base for any segmentation work.

## Recurring failures

- Recency-based handling of rejected admission victims hurt both hit rate and per-op time (cycle 1). The frequency-based admission gate already does the protective work, so extra recency rotation adds cost without benefit.
- Extra state in front of the admission sketch has not paid off. The doorkeeper (cycle 2) barely moved hit rate while adding handle state and per-put work, so one-hit pollution does not look like the limiting factor on the tested trace.
- Added bookkeeping has to earn its keep. Segmentation's small hit-rate gain (cycle 1) came with per-node and per-cache metadata growth, and the latency drop it showed is not explained by that bookkeeping.
- Changes that look behaviour-preserving may not be. Hoisting the hash out of the put path (cycle 2) moved hit rate, so the speedup cannot be counted as a pure cost change until equivalence is confirmed.

## Open questions

- Does any hit-rate gain hold beyond lirs workload-a? Every result so far comes from one trace, so no gain has been shown to generalise.
- Is segmentation's latency drop real or measurement variance? Its bookkeeping would predict a slowdown, not a speedup.
- Do the precomputed-hash helpers produce the same counter indices as the public sketch methods? The answer decides whether the hash-reuse hit-rate drop is a bug or a real trade-off.
- How much of the newest-key relink saving depends on how often accesses hit the newest key? A trace with a different share of such accesses would show whether the gain is structural.
- Is the doorkeeper's memory cost fully accounted for? Its storage does not appear in the metadata breakdown.
- Does the doorkeeper show any effect on a trace where one-hit keys dominate?

## Per-policy notes

- TinyLFU admission (frequency sketch, admission gate, doorkeeper): the gate appears to do the protective work already. Hash-reuse on the admission and sketch-update path is worth pursuing, but only after equivalence is verified. Front-end filters such as the doorkeeper have not helped on the tested trace.
- Segmented resident list (probation/protected split): the only policy change that moved hit rate, but the movement is small and unconfirmed on a second trace. Its metadata cost should be reduced via field packing before further builds on it.
- Victim and recency handling (MRU-tail rotation of rejected victims, newest-key relink skipping): rotation of rejected victims is a dead direction. Relink guards that preserve eviction order are a productive direction for speed.
