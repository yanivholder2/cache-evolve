# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants branched from the baseline, and each tested one axis on its own: residency structure (v001, W-TinyLFU window plus SLRU), sketch freshness (v002, periodic halving with capped counters), and hashing cost (v003, a single cheap hash per key). The winners were v001, which gave the clearest hit-rate gain, and v003, which gave the largest per-op-time gain. v002 also improved hit-rate, but it lost ground on median latency.

## Patterns

- The baseline had two separate weaknesses, and they map cleanly to the two winners. Hit-rate was held back by admission policy (strict admit-or-drop, a sketch that never forgets). Latency was held back by repeated SipHash. Neither winner's fix touched the other's axis, so the two should combine with little conflict.
- Both hit-rate fixes (v001, v002) gained most at the small cache size and much less at the large one. Admission contests at small capacity are where the baseline lost most.
- `get` p99 rose in v001 and in v003, for different reasons: extra region branching and relinks in v001, exposed sketch memory misses in v003. v001 and v002 both improved the `put` tail, but neither could explain why, so treat that as weak evidence. The get tail is now the main per-op-time weak spot across the cohort.
- Metadata cost was flat or slightly better, apart from v001's per-node region byte, which the padding turns into a full word.

## Open questions

- Do v001's window/SLRU and v002's aging add together, or does the window already absorb most of the stale-hot-key problem?
- Is v003's get-tail rise real, or benchmark noise? Would a cache-line-local sketch block fix it?
- Can v001's region tag be packed into spare bits to win back the per-node bytes?

## Continuation

Next cycle should start by merging v001's W-TinyLFU structure with v003's single-hash fast path. Then layer v002-style aging on top, stored as packed 4-bit counters so it costs no extra heap or sweep work. After that, target the get tail with a cache-line-blocked sketch, and remove v001's region byte. Do not try aging on its own again over the u32 counter layout. Its latency cost is already known.


### Cycle 0002

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


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
