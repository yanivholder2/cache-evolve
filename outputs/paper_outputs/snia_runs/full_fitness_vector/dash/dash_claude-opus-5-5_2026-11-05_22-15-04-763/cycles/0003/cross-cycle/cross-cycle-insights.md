## Recurring wins

- Retention-extending changes reliably raise hit rate: the two-hit gate (c1), MRU demotion into stash (c2, c3, also ungated), and probing overflow (c2). Every evaluated variant on this line improved hit rate on all trace/size cells except the one that removed first-hit reordering.
- hm_0 at the large size is consistently the most sensitive cell — biggest gains from gate and MRU demotion, biggest loss when retention is weakened. Use it as the primary signal for retention ideas.
- Cutting demotion cascades is the one lever that has recovered tail latency: probing overflow on top of gate + MRU demotion reversed most of the `put` p99 regression (c3).
- Reusing existing per-item metadata (e.g. the unused `lfu_counter`) adds state without growing metadata size.
- Current best base: gate + MRU demotion + probing overflow (c3 v002) — best combined hit rate and tail profile so far, still no Winner.

## Recurring failures

- No cycle has produced a Winner; the persistent blocker is `get` latency (esp. p50) regressing versus baseline once stash retains items longer.
- Latency fixes on the first-hit path don't pay: skipping the first-hit MRU move (c2) gave negligible p50 help, no p99 help, and lost hit rate.
- Lookup reordering doesn't pay: checking normal tier before stash (c3) left hit rate flat and worsened `get` p50/p99 — stash hits are too large a share of hits.
- Blaming the gate for `get` slowdown is wrong: ungated MRU demotion lands in the same slower `get` band (c3). Cost tracks stash-hit traffic and the promote/demote chain.
- Stacking promotion-placement and demotion-placement changes gives sub-additive hit-rate gains; overflow's value on top of MRU demotion is mostly latency, not retention.
- Build infra (Maven/Gradle HTTP 429) killed variants in c1; failures of that kind say nothing about the idea.

## Open questions

- Can the stash-hit promotion path (remove → promote → demote cascade) be made cheaper — skipped, deferred, or batched when not needed — without losing retention?
- Is the remaining gap to dominating baseline only `get` p50? How much is left after cascade reduction?
- Does probing overflow on the ungated MRU-demotion branch make that cheaper line competitive?
- Can the probing-overflow lookup penalty (extra bucket probed) be removed, e.g. by migrating items back to target when space frees?
- Is any residual `put` p99 regression driven by stash fullness or benchmark noise? Still not cleanly separated.
- H03-style balanced promotion between target/probing buckets was never evaluated in isolation.

## Per-policy notes

- Dash (stash + normal tier): working theory — a small normal tier gets flushed by weakly re-referenced items — has been supported every cycle. Hit-rate levers: admission gating, MRU demotion, overflow placement. Latency drivers: stash-hit promotion chain and demotion cascade count; stash occupancy pressure hurts `put` tails. Keep stash-first lookup order.
