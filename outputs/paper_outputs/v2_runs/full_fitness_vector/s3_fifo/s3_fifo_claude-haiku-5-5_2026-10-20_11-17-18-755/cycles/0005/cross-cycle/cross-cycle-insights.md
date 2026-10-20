## Recurring wins

- Fingerprint-keyed ghost queue: put-path benefit with no hit-rate change, seen in Cycles 0003, 0004, and 0005. Its cost is container-level bookkeeping, not per-entry size.
- Put-path edits that remove per-eviction or per-put key handling have the clearest mechanism. Hit rate stays flat when eviction and promotion logic is left alone.
- Whole-structure footprint is the right unit for metadata changes. Per-entry size is a poor proxy, because several changes shrank the entry or left it alone and still grew the container.
- Ghost population has a sweet spot below full capacity. Capping it below main-queue size (Cycle 0005) gave a small gain, while enlarging it to full capacity (Cycle 0004) lost hit rate.

## Recurring failures

- Enlarging the ghost toward full cache capacity: re-admits more keys to main without a matching reuse gain, and hit rate falls (Cycle 0004).
- Lowering the small-queue promotion threshold (`freq > 0`): moves singly-reused keys into main early, displaces useful residents, and costs hit rate with no attributable latency gain (Cycle 0001).
- Stacking the lowest-frequency victim choice on the enlarged small queue: lost hit rate and did not compose with the larger share (Cycles 0002, 0003).
- Side-array split of the frequency counter: entry bytes shrink, but the container grows and each hit pays an extra indexed read. Net footprint is a loss (Cycle 0004).
- Frequency aging on main-queue rotation: no hit-rate change and no attributable benefit on the one trace reported. Untested, not shown neutral (Cycle 0003).
- Sequence-tagged ghost queue: preserved policy but added per-eviction bookkeeping on the put miss path (Cycle 0002).

## Open questions

- Does the fingerprint ghost's put-latency gain hold across traces and capacities, and does its hash-set cost and linear ghost-hit scan matter in tail latency at larger sizes?
- Does the half-main ghost cap keep its small gain on other workloads, and does it still help once the ghost is stored as fingerprints? Is there a useful ghost size between half-main and full capacity?
- Do the small-queue share increase and the post-budget victim choice hold their small hit-rate gains across other traces and capacities? Are they in conflict, as the stacked result suggests?
- Does the id-indexed lookup keep its get and put speedup under a baseline rerun, and does the extra index state matter at larger sizes?
- Is the get-path latency shift real or shared measurement noise? It appears in variants whose get code is unchanged, and it is unexplained in most cases.
- Where is the dominant put miss-path cost, if the ghost position scan and per-eviction key handling are not it? Does the O(n) ghost pop-back still dominate once membership is O(1)?
- How often does the second-chance rotation budget run out under the evaluated workloads? Frequency aging and victim choice both depend on that.

## Per-policy notes

- Small queue: the promotion threshold (`freq > 0`) is a hit-rate loss. The size increase shows a small gain on one trace, and it does not combine with the victim choice. Validate before keeping either.
- Main queue: the rotation budget and victim choice are coupled. The budget's exhaustion frequency is unmeasured. Frequency aging has no measured effect yet.
- Ghost queue: fingerprint storage is the most consistent win on the put side. Enlargement toward full capacity is a failure. A capped population is a possible small gain. Sequence tagging and position-based removal have not shown a dominant cost.
- Metadata layout and lookup: id-indexed hash lookup removes per-put and per-hit scans with no hit-rate change, but its get-path gain still needs a baseline rerun. Side-array counter storage is a footprint loss.
- Cross-cutting: changes that touch overlapping state (ghost storage, lookup structures) have not been tested in combination. Their gains may not add.
