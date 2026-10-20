# Cross-cycle insights

## Recurring wins

- No cycle has shown a demonstrated hit-rate gain. The only pattern that has held up is that cost-side, eviction-preserving changes (resident-key lookup, membership-bit folding, early return on resident hits) keep landing hit-rate-neutral, as intended.
- Those cost claims (per-op-time, metadata-size) have never been reported, so they remain candidates, not wins.

## Recurring failures

- **Promotion bar (small queue):** relaxing it lost hits; raising it was neutral. The axis looks saturated on this trace and has not produced a gain in either direction.
- **Ghost re-admission:** widening the ghost window, routing ghost hits through the small queue, and giving re-admissions an extra main-queue pass or a frequency of 1 each lost hits or failed to help. No ghost-admission variant has improved hit-rate.
- **Main-queue retention and reinsert position:** keeping warm residents longer via a tail second pass lost hits, so the existing head reinsert is not leaving value on the table here.
- **Frequency axis:** raising the frequency cap and halving the decay step were both flat. This axis looks inert on this trace.
- **Small-queue sizing:** shrinking the queue was flat; an adaptive ±1 rule lost hits. The ghost-hit signal appears too noisy to steer sizing.
- **Unverified edits:** several variants could not be confirmed in their overlays, so flat or negative results may not reflect the intended change.

## Open questions

- Do the resident-key index, miss-path probe removal, and bit-folding reduce per-op-time or metadata-size? A rerun reporting both is needed before any ranking.
- Does removing the ghost-membership mirror in favor of a linear scan net out as a cost win, or does the miss-path scan regress per-op-time?
- Are the flat and negative results specific to this trace and capacity, or would any lever move hit-rate on another workload or size?
- Is there any hit-rate-moving lever left on this trace, or is the current S3-FIFO structure near its ceiling here?

## Per-policy notes

- **Small queue:** promotion bar and sizing levers are flat or negative; adaptive sizing is not yet a source of gains.
- **Ghost queue:** window size, admission routing, and mirror structure have all been probed without a gain; the mirror removal's cost effect is unmeasured.
- **Main queue:** frequency cap, decay, and reinsert position are flat or negative; retention changes have lost hits.
- **Resident-key index and metadata (cross-cutting):** the most promising cost-side direction, but unmeasured on per-op-time and metadata-size; overlay edits need confirmation before results are attributed.
- **Evidence base:** all cycles use one trace (lirs workload-a) at one capacity (2048).
