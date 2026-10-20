# Cycle 0007 Insight

## Cohort outcome

All three variants branched from cycle 0006's v001 (CLOCK). v001 replaced the per-bit sweep with a word-at-a-time scan. v002 collapsed the fill-phase double map probe into one `entry` call. v003 let the duel compare the newcomer against the colder of two clear-bit CLOCK candidates. All three are listed as Winners, but none moved a headline axis forward. v001 and v002 kept hit rate identical and made latency slightly worse. v003 lowered hit rate.

## Patterns

- Both latency fixes optimised paths the trace rarely takes. Long CLOCK sweeps are rare, because newcomers enter with a clear bit. The fill phase runs only once, during warm-up. Before targeting a path, measure how often steady state actually reaches it.
- Fixed per-call overhead and struct layout matter more than worst-case work. Even a small fixed cost, such as mask math, an out-of-line call or one extra struct field, can outweigh a rarely used saving. Growing `TinyLfuCache` past two cache lines may be what slowed down an unchanged `get`.
- `get` latency moved on all three variants, even though no variant changed `get`. The p99 tails in particular can't separate small edits from noise or layout shifts. A gain only counts if a mechanism explains it and it repeats against the parent side by side.
- Using more candidates to choose the victim hurt again, as cycle 0002's LRU-tail sampling did. Taking the minimum of two estimates loosens admission. Leaving the hand parked next to the slot just filled makes new admits the next victims, so they evict each other. The strict single-victim duel at the door is still the best admission rule for workload-a.
- Both behaviour-preserving variants kept hit rate exactly, so the CLOCK base is stable to refactor.

## Open questions

- Does `put` p99 for the CLOCK base come from the sweep at all, or from the full-miss map work: a `get` probe on rejection, then `remove` and `insert` on admission, each re-hashing the key?
- Would a hash-aware/raw-entry API that reuses the precomputed `h` across those three map operations cut steady-state `put` cost?
- Would a two-candidate scheme help if the admission threshold stayed tied to `a` and the hand moved past both candidates? Or should this line be closed?

## Continuation

Keep cycle 0006's v001 (CLOCK) as the base, and don't adopt any of this cycle's variants. Drop sweep micro-optimisation, fill-phase map tweaks and min-of-candidates admission. Don't add struct fields to the cache's hot layout. If latency is the target, aim at the steady-state full-miss map path, for example by reusing the precomputed hash across get/remove/insert. Count how often each path runs before targeting it, and judge success by medians repeated against the parent side by side, not by single p99 readings.
