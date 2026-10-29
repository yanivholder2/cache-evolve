# Cycle 0002 Insight

## Cohort outcome

Every variant was a follow-up to a cycle-0001 open question. v001 swapped the hasher on CLOCK, v002 added a GCLOCK counter in CLOCK's padding byte, and v003 ported SLRU onto the `u32` node with an in-link flag. All three are Winners, but only v003 got a clear gain without a matching cost. It kept SLRU's full hit-rate benefit on a compact node, and its remaining price is confined to put.

## Patterns

- Folding policy state into existing bits now works. A segment flag in a link's high bit (v003) and a counter in the padding byte (v002) both kept the node at its compact size. So the metadata objection to frequency- or scan-aware policies is settled. What's left to fix is latency.
- Hit-rate gains came with put-path costs in both policy variants. In SLRU the cost is the promote/demote double relink plus branching per segment. In GCLOCK it is sweep length multiplied by decrement-by-one aging. Neither added meaningful cost to get p50, so the work is in the put/eviction path, not the hit path.
- SLRU beat GCLOCK on hit rate: it improved every trace at both sizes, while GCLOCK's gain was mostly hm_0 at the large size. Neither variant fixed web_0 at the large size, so plain frequency counting is not what that trace needs.
- The hash-map lever failed again, and this time get got worse. Making the hash cheaper doesn't help. The probe appears to be bound by memory and sensitive to hash distribution on structured block keys. This is inferred, since probe lengths aren't measured.

## Open questions

- Can SLRU's put cost be cut? For example, avoid the second relink on promotion, give puts to existing keys a cheaper path, or remove the per-segment head/tail branching.
- Was GCLOCK's get slowdown real, or an artifact of the long put sweeps interleaved with gets in the benchmark?
- Would a denser or smaller table, such as narrower values or a better-mixing finaliser, move get latency where a cheaper hash did not?
- What does web_0 at the large size need, given that neither recency segmentation nor small counters fixed it? Possibly ghost/history entries or an adaptive split.

## Continuation

Make v003's flag-packed SLRU on `u32` nodes the default base, and next try to cut its put-path cost: cheaper promotion/demotion and a fast path for updating existing keys. Drop cheaper-hash ideas. Drop GCLOCK with decrement-by-one aging unless aging is moved off the eviction path, for example by bulk halving. If you go after get latency, try table density or hash quality, not hash speed.
