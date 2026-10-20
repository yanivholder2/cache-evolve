## Recurring wins

- Put-path bookkeeping simplification is the one area with attributable gains across cycles: a ghost-membership hash index (Cycle 0001) and fingerprint-keyed ghost entries (Cycle 0003) both reduced put cost without changing policy.
- Pure data-structure swaps that leave eviction logic alone keep hit rate unchanged, so they are safe to evaluate on latency alone.
- Small hit-rate gains from the larger small-queue share and the post-budget victim choice (Cycle 0002) have not yet repeated; treat them as candidates, not wins.

## Recurring failures

- Promotion-threshold changes in the small queue (`freq > 0`, Cycle 0001) moved singly-reused keys into main too early, displacing useful residents and costing hit rate with no attributable latency gain.
- Stacking the lowest-frequency victim choice on the 20% small queue (Cycle 0003) lost hit rate; the victim change has little to act on once the small queue is larger.
- Sequence-tagged ghost entries (Cycle 0002) preserved policy but added per-eviction bookkeeping on the put miss path.
- Bounding eviction scans (Cycle 0001) traded put cost for forced evictions of hot entries.
- Winner labels have repeatedly outrun the evidence: several mechanism claims rest on hypothesis text because overlay source could not be inspected, and single-trace hit-rate moves were not replicated.

## Open questions

- Is the get-path latency shift seen in every cycle real, or shared environment or measurement noise? No variant's diff touched `get()`, so re-measure a baseline before crediting any get-path change.
- Does the larger small-queue share hold across other traces and capacities, or is the gain specific to the trace and size tested?
- Does the post-budget victim choice ever help, and under what rotation-budget exhaustion rate?
- Does the fingerprint ghost keep its put-latency gain on more traces, and does its linear ghost-hit scan show up in tail latency at larger sizes?
- Where is the dominant put miss-path cost, if the ghost `position` scan is not it?

## Per-policy notes

- Small queue: keep the frequency threshold at its existing value; the `freq > 0` promotion change lost hit rate. The share increase (Cycle 0002) needs validation across traces and capacities before adoption.
- Ghost queue: hash-indexed membership and fingerprint keys reduced put cost; sequence tagging added cost without a policy gain and should be dropped. Fingerprint ghost ghost-hit tail latency is unverified.
- Main queue: the second-chance rotation budget is a useful bound on put cost but its exhaustion frequency is unmeasured. Frequency aging (Cycle 0003) showed no hit-rate change on one trace, so its effect is untested; hold it until a trace shows it changing evictions.
- Eviction scan: bounding scans lowers put cost but forces eviction of hot entries; any cap should be measured against hit rate, not latency alone.
