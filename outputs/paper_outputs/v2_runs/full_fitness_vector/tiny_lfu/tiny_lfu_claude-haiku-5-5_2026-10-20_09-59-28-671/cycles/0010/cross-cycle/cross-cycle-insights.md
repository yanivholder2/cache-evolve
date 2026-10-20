# Cross-cycle insights

## Recurring wins

- Constant-factor, eviction-preserving work is the reliable source of gains. Hit rate stayed at the parent's value in most cases while per-op time moved (hash reuse, seed caching, single-loop updates, get-path short-circuits).
- Keeping per-row sketch seeds in a module-level `const` table, not per instance, kept the speedup without growing structs. Hit rate stayed flat.
- Packing protected-segment state into existing node fields cut metadata without changing hit rate.
- Collision-spread changes in the sketch are the only lever that has moved hit rate upward. Conservative updates and a wider, shallower sketch each showed small gains. Narrow counters showed a gain once and a loss once, so the result is unsettled.
- Index-preserving changes keep hit rate; changes that alter which keys collide move it.

## Recurring failures

- Changes to admission or aging did not raise hit rate: a doorkeeper, shorter frequency aging, a recency window in front of the gate, and admission tie-breaking. The frequency gate appears not to be the limiting factor on this workload.
- Shrinking or capping the protected segment cost hit rate. Protected-tail demotion alone left hit rate unchanged.
- Recency rotation of rejected victims hurt both hit rate and per-op time.
- Per-row work added on get or put paths (per-row indexing, per-row column lookups, extra list moves) cost per-op time and gave back earlier speedups.
- Single-run timing shifts were repeatedly unexplained or not reproduced. Treat put-side timing claims as unverified until repeat runs confirm them.

## Open questions

- Do the collision-spread gains (conservative update, row/column trade, narrow counters) hold on a second trace and at other cache sizes? All evidence so far is one trace at one size.
- Is the narrow-counter result a sizing error? The u8 variant's sketch came out at half its intended size, so it needs a re-run before the direction is judged.
- Do the const-table seeds produce the same counter indices as the public per-row path for every key, or only in aggregate hit rate? This has been open since the early cycles.
- Can the public per-row indexing be made cheap enough to keep its fidelity without losing the speed gain?
- Does the get-path MRU short-circuit and the single-loop conservative update's put-tail gain survive repeat runs?
- Does the one-hit-wonder-heavy trace ever exist in this set? The doorkeeper was never tested where it could matter.
- Cycle 0001 labels its trace "lirs workload-a"; later cycles say "workload-a". Confirm these name the same trace.

## Per-policy notes

- TinyLFU-style admission (count-min sketch with a frequency gate): hit-rate movement has come only from sketch collision spread. Admission-side and aging-side changes have not helped.
- Segmented LRU (protected and probation segments): the protected segment's capacity matters more than its tail turnover. Reducing capacity hurt; forcing demotion more often did not pay off.
- Sketch layout (rows, columns, counter width, cache-line grouping): layout changes that preserve collision patterns hold hit rate. Those that change collisions can gain or lose and need trace-level confirmation.
- Metadata: packing into existing fields is safe. Per-instance seed storage added growth and should stay in constant tables.
