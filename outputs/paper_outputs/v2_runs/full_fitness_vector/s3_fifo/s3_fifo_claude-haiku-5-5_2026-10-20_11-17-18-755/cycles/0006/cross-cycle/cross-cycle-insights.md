# Cross-Cycle Insights

## Recurring wins

- Fingerprint-keyed ghost queue has given a put-path benefit with no hit-rate change in three cycles (0003, 0004, 0005). Its cost is container-level bookkeeping, not per-entry size, and its ghost-hit path scans linearly.
- Id-indexed lookup and index consolidation reduced container state and put cost while leaving eviction, ghost, and promotion logic untouched (0005, 0006). Hit rate stayed flat in both. These are the cleanest structural gains so far, though the get-path speedup claim still needs a baseline rerun.
- Edits that preserve policy have produced unchanged hit rate, which is useful evidence that the change was structural and not a policy shift.

## Recurring failures

- Enlarging the ghost queue toward full cache capacity lost hit rate (0004). It re-admitted more keys to main without a matching reuse gain.
- Stacking a lowest-frequency victim choice on the larger small queue lost hit rate (0003). The rotation budget likely runs out less often once the small queue is larger, so the victim change has little to act on.
- Ghost sequence-tagging added put-side bookkeeping without a policy gain (0002).
- Splitting the per-entry frequency counter into a side array grew the whole structure and added an indexed read on hits (0004). Shrinking `Entry` did not shrink the footprint.
- Lowering the small-queue promotion threshold to `freq > 0` moved singly-reused keys into main early, displaced useful residents, and showed no attributable latency gain (0001).
- Get-path timing moved in variants whose get code was unchanged (0001 through 0006). Treat these shifts as shared noise or environment drift, not as variant effects.

## Open questions

- Does the fingerprint ghost's put-latency gain hold across other traces and capacities, and does its hash-set and linear ghost-hit cost stay acceptable at larger sizes?
- Can the half-main ghost cap's per-eviction drain be amortized (for example, batch compaction) so its small hit-rate gain survives without the put cost? Does it still help once the ghost stores fingerprints?
- Does the small-queue share increase help on its own? Its hit-rate effect has only appeared bundled with the victim change, and standalone runs showed no movement.
- How often does the second-chance budget run out under the evaluated workloads, and does the victim choice help in that regime?
- Is the get-path shift real and reproducible under a baseline rerun with no code change?
- Do the index and ghost edits compose? None has been tested in combination.

## Per-policy notes

- Small queue: The promotion threshold change (`freq > 0`) was a hit-rate loss with no attributable gain; drop further tuning there. The share increase alone showed no hit-rate movement and only modest, single-run latency movement, so it lacks a Winner-grade case.
- Ghost queue: Fingerprint storage is the most consistent put-side win. Enlargement toward full capacity hurt hit rate. A half-main cap gained a small amount but adds put cost through its drain. Sequence-tagging added cost without benefit. Position-based removal does not appear to be the dominant put cost.
- Main queue: The post-budget lowest-frequency victim choice lost hit rate when stacked on the larger small queue, and its recovery benefit is untested in a regime where the budget runs out often. Frequency aging on rotation left hit rate unchanged on the one trace reported and has not yet been shown to change any eviction.
- Lookup and metadata (cross-policy): Index consolidation and id-keyed lookup are the strongest structural gains. Per-entry packing changes have not paid off; the cost is in container-level state.
- Caveat: Several mechanism claims rest on hypothesis text and fitness deltas because overlay source was not inspected, so attribution remains provisional.
