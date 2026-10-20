## Recurring wins

- No hit-rate gain has been demonstrated in any cycle. Every Winner so far is hit-rate neutral against its parent.
- Behavior-preserving cost refactors have held hit-rate flat across many cycles (resident-key probes and index, ghost-lookup restructuring, bit-folding, early return on resident hits, hasher swap, pre-filter). Their intent is cost, not hit-rate, and this pattern has held up.

## Recurring failures

- Ghost re-admission has lost hits in every direction tried: widening the ghost window (0003), routing ghost hits through the small queue (0004), giving re-admissions an extra main-queue pass (0007), and carrying frequency 1 into main on re-admission (0007).
- Promotion bar changes have not helped: relaxing it to one hit lost hits (0001), and raising it to three was neutral (0003). Moving warm small-queue keys earlier changed timing but not residency (0008).
- Main-queue retention changes lost hits: keeping frequency-positive victims via a tail second pass instead of head reinsert (0006).
- Frequency-axis tweaks are flat: raising the main-queue frequency cap (0002) and halving the decay step (0005).
- Sizing changes are flat or negative: shrinking the small queue (0001) was neutral, and making the small-queue size adaptive to ghost hits (0005) lost hits.

## Open questions

- Per-op-time and metadata-size have never been reported, so no cost-side candidate can be ranked against the baseline. Does the resident-key index or any cost refactor actually reduce cost?
- Have the overlay edits been confirmed in the built code? Several cycles lack verification, so flat or negative results may not reflect the intended changes.
- Are the flat and negative results (frequency, sizing, ghost routing) specific to this trace and capacity, or would they change on other workloads or capacities?
- Does a larger or structurally different change to ghost admission or sizing move hit-rate, given that single-constant tweaks have not?

## Per-policy notes

- S3-FIFO, small queue: the two-hit promotion bar appears to filter usefully (relaxing it lost hits), but raising it further is neutral. Promotion timing alone does not change residency.
- S3-FIFO, main queue: the frequency cap and decay are inert on this trace. Retention via tail reinsert lost hits, so head reinsert looks adequate here.
- S3-FIFO, ghost queue: the most-probed component, and every admission-side change so far has lost hits or failed to help. Ghost membership and lookup structure are the cost-side work to watch, not admission rules.
- S3-FIFO, sizing: the fixed split is hard to beat. Shrinking was neutral, and adaptive sizing from ghost hits was noisy and lost hits.
- Evidence for all of the above is one trace (lirs workload-a) at one capacity (2048).
