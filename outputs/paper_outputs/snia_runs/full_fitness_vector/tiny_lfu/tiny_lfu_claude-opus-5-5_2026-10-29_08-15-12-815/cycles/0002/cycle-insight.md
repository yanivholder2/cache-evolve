# Cycle 0002 Insight

## Cohort outcome

Two variants branched from cycle 0001's fast-hash winner. v001 merged in the W-TinyLFU window plus SLRU structure, and v002 replaced the sketch with a cache-line-blocked one using packed 4-bit counters and aging. The third, v003, added an adaptive hill-climbing window on top of cycle 0001's W-TinyLFU. v001 and v002 won because each improved hit-rate and per-op-time together. v003 gained hit-rate only at the small cache size and paid for it in latency on every op.

## Patterns

- The two winners changed separate parts of the cache: v001 changed residency and the map key, v002 changed sketch storage. Both still improved hit-rate and latency together, so the earlier tradeoff between the two axes is gone. Each attacked a different cost that the cycle-0001 fast-hash parent still had: v001 removed the second hash into the map, and v002 removed the scattered sketch cache misses. The get-tail weakness flagged last cycle is resolved in both.
- Most hit-rate gains are still at the small cache size. Each hit-rate mechanism (window/SLRU, aging, adaptive window) also showed a small slip in one large-cache configuration. This is consistent across the cohort: at large capacity, mechanisms that forget or bypass frequency evidence start to cost hit-rate.
- Packing bits paid off twice. Sentinel-tagged links removed the region-tag cost and shrank the node (v001). Nibble packing shrank the sketch heap and made aging cheap (v002). Running aging over packed counters did not bring back the latency cost cycle 0001 saw.
- Runtime adaptivity (v003) cost latency on every op through rebalance work and bookkeeping in `get`. Its hit-rate gain may come partly from the path that fills the window without a frequency contest, not from window size alone.
- There are unmeasured costs: v001 stores an extra hash in each map entry, and v001 and v003 grew the fixed cache struct. The current metadata metric does not capture either.

## Open questions

- Do v001's residency/map-key changes and v002's blocked aging sketch combine additively, as the previous merge did?
- What causes v002's large-cache slip: 4-bit saturation, the fixed sample-period multiplier, or correlated collisions within a block?
- Does v003's small-cache gain survive once the frequency contest is restored on the window-fill path?

## Continuation

First, merge v001 and v002 (window/SLRU with a prehashed map key, plus the blocked 4-bit aging sketch). Then tune the aging sample period, or the counter width, per cache size to fix the large-cache slip. Also apply the `NIL` sentinel encoding to `next`, and count the per-entry map hash in the metadata figures. Do not pursue the hill climber in its current form. Try it again only with the contest restored and a climber that stops at its step floor, and only if the merged variant still leaves a gap at the small cache size.
