# Variant v001 Insight

## Helped

- hit-rate: Routing all eviction through the probationary LRU head keeps one-shot scan keys from displacing the working set, so re-referenced keys (promoted to the 4/5-sized protected segment on their second touch) survive across scans on lirs:multi1 where pure recency retained nothing.

## Hurt

_(No other fitness dimensions were measured; per-op time and metadata footprint of the extra head/tail pair and per-node segment tag are not exercised by this vector.)_

## Takeaway

On lirs:multi1, the win is structural rather than tuning: gating promotion on a second hit and pinning eviction to the probationary head is what separates "scan-resistant" from baseline LRU, not the specific 80/20 split. The arena-slot-reuse pattern carried over cleanly — segmenting only adds a second pair of head/tail pointers and a one-byte tag, so the existing P1/P2/P3 invariants still hold without a free list. Because the variant's hit-rate was sampled at a different cache size than the baseline's, this run shows that SLRU produces a non-zero hit rate on this trace at all, but does not by itself prove a same-size delta over plain LRU.
