## Recurring wins

- Bounding the eviction scan is a recurring theme. The scan cap in the first cycle traded put cost for forced evictions of hot entries. The post-budget victim choice in the second cycle is aimed at that same cost, and its hit-rate recovery is the most promising follow-up. Neither is confirmed yet.
- Changes that leave policy intact and touch only the put path give predictable, explainable cost effects. The hash-indexed ghost membership test in the first cycle is the clearest example. Hit rate was unchanged, as expected for a pure data-structure swap.
- Hit-rate gains so far have been small and came from a single trace at a single capacity. None is robust yet.

## Recurring failures

- Tuning the small-queue promotion threshold (`freq > 0`) moved singly-reused keys into main too early and displaced useful residents. It produced no attributable latency gain and a hit-rate loss. Do not retry it.
- Sequence-tagging the ghost queue kept policy identical but added per-eviction bookkeeping on the put miss path. It is a repeat of the same pattern: policy-neutral bookkeeping that costs put latency without a hit-rate return. Do not retry it.
- Get-path latency shifts appear across variants even though `get()` was not modified in any of them. Crediting these shifts to any variant's diff has repeatedly been unsupported.
- Mechanism claims have leaned on hypothesis descriptions because the overlay source could not be inspected. Winner labels are stronger than this evidence supports.

## Open questions

- Is the get-path latency shift real and reproducible, or a shared environment or measurement artifact? Does it appear with no code change at all? A baseline rerun is needed before any get-latency claim.
- Does the larger small-queue share hold across other traces and capacities, or is its gain specific to the tested workload and size?
- Does the lowest-frequency victim choice keep its hit-rate and put-tail benefit when the rotation budget runs out more often? How often does the budget run out under the evaluated workloads?
- Where is the dominant put miss-path cost, if not the ghost `position` scan? Does the O(n) pop-back from the ghost queue now dominate, given that membership is O(1)?

## Per-policy notes

- S3-FIFO small queue: the promotion threshold is exhausted as a tuning axis; leave `freq > 0` alone. Queue share is the open lever, with a hit-rate gain on one trace that needs validation across traces and capacities.
- S3-FIFO ghost queue: the hash index for membership is a cost-side win with no policy effect. Sequence tagging and the position-based removal are not where the put cost sits. The remaining candidates are pop-back cost and maintenance overhead.
- S3-FIFO main queue: second-chance rotation has a fixed budget. Once it is exhausted, a lowest-frequency victim choice is the most promising bounded fallback. Its benefit depends on how often the budget runs out, which is still unmeasured.
- Cross-policy measurement: get-path timings should be treated as noise until a baseline rerun separates them from any real effect.
