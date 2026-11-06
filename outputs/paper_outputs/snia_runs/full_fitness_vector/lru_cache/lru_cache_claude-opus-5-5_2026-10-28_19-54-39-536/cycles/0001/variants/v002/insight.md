# Variant v002 Insight

## Helped

- metadata-size: Dropping the `prev`/`next` link indices from `Node` and adding only a `referenced: bool` (padded into the existing key/value layout) halved per-node overhead. Replacing `head`/`tail` with a single `hand` also shrank the `LruCache` struct.
- per-op-time (put p50): A put that evicts now overwrites the slot under the hand in place, with no `unlink`/`push_back` relinking. The typical insert path got cheaper.
- per-op-time (get p50/p99): A hit now writes one byte on a node that is already loaded, so get became slightly faster at both median and tail. The gain is far smaller than the "about six scattered writes" rationale predicted, which means the `HashMap` lookup, not list maintenance, dominates the baseline get.
- hit-rate: Inserting fresh keys with `referenced = false` means a key touched only once is evicted on the next sweep, before any key that has been re-referenced. This gives mild scan resistance. It lifted hm_0 at both cache sizes and web_0 at the small size, outweighing the small loss below.

## Hurt

- per-op-time (put p99): The CLOCK sweep's length depends on how many consecutive slots under the hand have their reference bit set. Under hit-heavy phases, an eviction can walk a long run of slots, so put tail latency got worse even as the median improved.
- hit-rate (web_0, large cache): A single reference bit can't tell a key hit once apart from one hit many times, and victim order follows slot position rather than recency. At the large size on web_0 that cost a little versus strict LRU.

## Takeaway

Lazy promotion technically beat the get p50 target, but only by a hair, so relinking was never the main cost of a hit. The `HashMap` probe is the next thing to attack if get latency is the goal. The clearer wins from the CLOCK rewrite are the halved node metadata and a cheaper typical put. The price is a put tail that now depends on the reference-bit density the sweep encounters, so a follow-up should bound or amortise the sweep.
