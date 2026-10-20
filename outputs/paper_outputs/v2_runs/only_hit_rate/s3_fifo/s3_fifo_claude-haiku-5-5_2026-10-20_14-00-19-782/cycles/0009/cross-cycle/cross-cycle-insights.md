## Recurring wins

- Behavior-preserving cost refactors (resident-key indexing, hasher or fingerprint swaps, folded membership bits, early returns) have repeatedly held hit-rate flat, as intended. They are safe to land, but none has yet shown a measured cost gain.
- The only hit-rate movement in the ledger came from a ghost-side change: feeding main-queue evictions at zero frequency into the ghost queue. It is marginal and rests on one trace at one capacity.

## Recurring failures

- Relaxing the small-queue promotion bar lost hits, and tightening it to three hits was neutral. The promotion bar looks saturated on this trace, and timing-only changes to warm-key promotion did not alter residency.
- Ghost re-admission has been pushed several ways and has not paid off as a standalone lever: widening the ghost window lost hits, routing ghost hits through the small queue lost hits, and giving re-admissions an extra main-queue pass or frequency of 1 did not help. Re-admitted keys appear to displace resident entries.
- The frequency axis is inert here. Changes to the main-queue frequency cap and decay step were flat across cycles.
- Sizing levers have not helped: an adaptive small-queue size, driven by ghost hits, lost hit-rate, and the ghost signal looks too noisy to steer the split.
- Retaining warm main-queue residents longer, via a tail second pass instead of head reinsert, lost hits. The existing reinsert position is not leaving value on the table.

## Open questions

- Does the main-eviction ghost feed hold on other traces and at other capacities, and does its gain survive the unchanged ghost scans?
- Do the cost-side refactors (resident index, hasher change, bit folding, early return, ghost mirror removal, ghost pre-filter) reduce per-op-time or metadata-size? No cycle has reported either dimension, so none can be ranked.
- Is the ghost linear scan the dominant miss-path cost? Any cost gain from indexing is bounded until it is addressed.
- Were the intended overlay edits actually present in the built code for each variant? This verification gap has persisted across most cycles, and several flat or negative results cannot yet be tied to their hypotheses.

## Per-policy notes

- S3-FIFO, small queue: promotion-bar and promotion-timing changes have been flat or negative. Routing re-admissions through this queue lost hits.
- S3-FIFO, main queue: frequency cap, frequency decay, and reinsert-position changes have been flat or negative. Feeding main evictions into the ghost queue is the one change with a measured gain.
- S3-FIFO, ghost queue: window size, admission routing, and re-admission frequency have not helped. Ghost-side changes that alter which keys enter the ghost queue, rather than how re-admissions are handled, are the only ones that have moved hit-rate.
- S3-FIFO, resident and ghost indexes: all changes here are cost-side and have held hit-rate flat. Their per-op-time and metadata claims remain unmeasured.
- Evidence across all policy groups comes from one trace (lirs workload-a) at one capacity (2048).
