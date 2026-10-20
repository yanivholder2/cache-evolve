## Recurring wins

- Behavior-preserving rewrites that remove a specific O(n) cost (scan consolidation, hash-index membership, O(1) ring-buffer ghost) are the only changes with a credible mechanism for their gains. Hit rate stays identical and `put`/`get` cost drops.
- Targeting the actual cost source works. Ghost maintenance was the `put` tail driver. Main-queue eviction and membership checks were not.
- A smaller small queue (10% then 5%) gave small hit-rate gains twice, from different parents. The curve is flattening.
- Keeping the small queue as a strict one-hit-wonder filter preserves hit rate.

## Recurring failures

- Loosening main admission has hurt in every cycle. Examples: lower promotion threshold, larger small queue, freq head start for main-evicted keys, longer ghost history, and recycling main evictions into the shared ghost.
- Ghost-size and ghost-policy tuning has repeatedly failed. The baseline looks near a sweet spot, and the early separate-ghost gain is doubtful.
- Rotation caps and CLOCK-style rewrites of main eviction gave nothing, because the freq counter already bounds rotation.
- Latency-only wins with no mechanism keep getting selected: one-constant edits with large drops, and `get` shifts in variants that never touched `get`. Treat these as noise.
- Infrastructure failures (Maven 429) can hide ideas without testing them.
- Refactors have deleted tests, and per-entry `metadata-size` hides the growth in whole-cache footprint from indexes and rings.

## Open questions

- Do the 5% ratio and the other small hit-rate gains survive repeated runs, other traces, and other cache sizes? All evidence comes from one trace at one size.
- Is the `get` p50 shift real, or a layout effect? Does the combined index + ring + scan stack regress `get`?
- What is left in the `put` tail now that ghost trimming is O(1)? Candidates are hash-map operations and the small-to-main move.
- What is the real memory cost of the index and ring?
- Is any special handling of main-evicted keys worthwhile? It would need an O(1) tagged-ghost design with evidence of faster reuse.
- Can the deleted tests be restored without losing gains?

## Per-policy notes

- S3-FIFO (the only policy so far):
  - Current base stack: scan consolidation, hash-index membership, O(1) ring ghost, and a 5% small queue (pending confirmation).
  - The promotion threshold stays at baseline. The main-eviction logic is settled and should be left alone.
  - Next gains should come from the remaining `put` tail outside ghost trimming.
  - Judge on hit rate over multiple traces, plus latency changes with a clear mechanism.
