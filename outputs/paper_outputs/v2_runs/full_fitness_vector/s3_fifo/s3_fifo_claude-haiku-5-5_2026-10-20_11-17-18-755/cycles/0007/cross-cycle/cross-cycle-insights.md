## Recurring wins

- Fingerprint-keyed ghost storage improved put-path cost in three separate cycles (0003, 0004, 0005) with no hit-rate change. It is the most consistent attributable benefit so far, though it has not been checked across traces or capacities.
- Id-keyed indexing in place of per-queue key scans recurred across cycles (0005, 0006, 0007). The merged single index (0006) is the cleanest case: lower container state and put cost, with eviction, ghost, and promotion logic untouched, so hit rate held.
- Batched ghost maintenance (push-count window with pruning, 0007) addresses the ghost drain cost directly, off the per-eviction path. It is the only edit that moves hit rate in a favorable direction without a clear put-side cost, but it rests on one trace.
- Policy-preserving changes to metadata and bookkeeping consistently leave hit rate unchanged, so they are safe to test, but their benefit is in cost rather than hit rate.

## Recurring failures

- Enlarging the ghost toward full cache capacity (0004) admitted more keys back to main without a matching reuse gain and lowered hit rate.
- Storing the frequency counter in a parallel side array (0004) grew the container more than it shrank the entry, a net footprint loss, and added a get-path cost.
- Sequence-tagging ghost entries (0002) added per-eviction bookkeeping on the put miss path without a policy gain.
- Lowest-frequency victim choice stacked on the larger small queue (0003) lost hit rate on the trace tested. Its earlier gain (0002) did not survive the combination.
- Promoting on any reuse (small-queue threshold `freq > 0`, 0001) moved singly-reused keys into main early, displaced useful residents, and showed no attributable latency gain.
- Standalone small-queue share increases (0006) left hit rate unchanged; the earlier hit-rate movement was attributed to the bundled victim change, not to the share.
- Frequency aging on main-queue rotation (0003) showed no hit-rate change on the one trace reported, so it has no demonstrated benefit.

## Open questions

- Do the fingerprint ghost's put-latency gain and the merged index's cost benefit hold across other traces and capacities, and does the hash-set and index state stay acceptable at larger sizes?
- Does the ghost window (0007) keep its hit-rate gain on other workloads, and can it be combined with fingerprint storage without losing its drain-cost advantage? Cycle 0006 showed the fingerprint ghost plus a half-main cap eroding put-side gains.
- Do the frequency-counter relocation (0007) and the index change compose, or do they conflict on the same record? Is the counter's container-size effect actually zero under whole-structure measurement?
- How much of the index variant's put gain comes from the index itself versus inherited ghost behavior? An ablation would separate them.
- Get-path timing has moved in variants whose get code was unchanged (0001 through 0007). Is this shared environment noise? Re-run a get-path baseline before crediting any get-latency change, including the index speedups.
- How often does the rotation budget run out, and does the victim choice matter once the small queue is enlarged? The latter has not been tested in isolation.

## Per-policy notes

- Small queue: the promotion threshold (`freq > 0`) hurt hit rate without attributable benefit; drop further tuning there. Share increases alone have been neutral on hit rate; their latency movement is single-run and unconfirmed.
- Main queue: second-chance rotation budget and lowest-frequency victim choice have only shown hit-rate gains on single traces and lost them when stacked on the larger small queue. Frequency aging has no demonstrated effect yet.
- Ghost queue: fingerprint storage is the strongest put-side direction. Enlarging the ghost to full capacity hurts; capping or windowing it is more promising, but its per-eviction drain cost needs separate measurement. Sequence tagging is not worth pursuing.
- Frequency counter and metadata: relocating the counter or storing per-entry data in side arrays has not shown a net footprint saving. Judge these changes by whole-structure size and get-path cost, not `Entry` size.
