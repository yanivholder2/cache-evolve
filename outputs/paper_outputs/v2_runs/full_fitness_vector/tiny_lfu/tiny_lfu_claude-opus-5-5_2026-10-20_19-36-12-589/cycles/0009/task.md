# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0009

## Winners

- v001
- v003
- v002

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0009/hypotheses/H03.md

## Prior cycles digest

### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

The cohort moved from the sketch to the structures around it. v001 replaced the LRU list with CLOCK reference bits. v002 replaced the std map with a custom index table that stores only node indices. v003 added a 10% admission window and folded in the fused sketch increment. All three are listed as Winners, but only v001 is a clean step forward: smaller per-entry metadata with hit rate unchanged. v003's latency gain comes from the fused increment, not the window. v002 brought no gain.

## Patterns

- Admission doesn't depend on exact recency order. CLOCK's coarse second-chance order left hit rate flat, so exact LRU position was never giving the duel any useful information. Per-node links can go without losing quality.
- Windows are now ruled out at both sizes. The ~1% window (cycle 0005) and the 10% window (this cycle) both lowered hit rate. Workload-a rewards strict frequency filtering at the door. Any capacity given to unfiltered arrivals goes mostly to one-hit keys.
- Median `get` cost doesn't sit in relinking or in hashing twice. Removing relinks (v001) and removing the duplicate hash (v002) both left the `get` median unchanged. The only change that moved latency was the fused register-held sketch increment (v003). That suggests cycle 0005's failure to carry this gain over was bench drift, not a real regression.
- Tail costs show up on the rare paths. CLOCK's unbounded sweep, and v002's second probe plus re-hashing during backward-shift deletion, both made `put` p99 worse. Structural changes need to keep eviction-path work bounded and free of extra hashing.
- A custom index that stores only node indices does worse than SwissTable. Every probe then has to load from `nodes`, which costs more than the hash it saves.

## Open questions

- Does word-at-a-time sweeping (`trailing_ones` over a `u64` of ref bits) bring CLOCK's `put` tail back to LRU's O(1) level?
- Would an index that stores hash tags or home slots beat std `HashMap`, or is the map simply not the bottleneck?
- With recency structure, sketch accuracy and duel inputs all used up, is workload-a's hit rate at its ceiling for this design family?

## Continuation

Use v001 (CLOCK) as the new base. Merge in v003's fused increment without the window, and bound the sweep with word-level bit scanning. Confirm the latency side by side against both parents. Drop admission windows of any size, and drop index-only custom hash tables. Treat further hit-rate work as low-yield. Prefer metadata and tail-latency reductions that keep the duel and the hit rate unchanged bit for bit.


### Cycle 0007

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


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

Each variant was on the CLOCK line and tested one lever. v001 keyed the map by the precomputed hash so each key is hashed once. v002 doubled the sketch width again. v003 put a periodically cleared doorkeeper in front of the sketch. All three are listed as Winners, but none moved a headline axis forward. v001 kept hit rate identical but slowed down a little. v002 gained a hit-rate amount no bigger than noise and paid for it in median latency. v003 lowered hit rate and raised latency.

## Patterns

- Hashing is not the cost. Cycle 0006 removed a duplicate hash and this cycle removed the map-side re-hashes, and neither changed latency. For `u64` keys the Fx-like hash is nearly free, and any guard compare or extra branch on the hit path costs more than it saves. Drop hash-count reduction as a latency lever.
- The sketch's memory footprint is now the clearest per-op cost. This is the first time a median regression has a clear mechanism behind it: the doubled table no longer sits comfortably in L1/L2, so `get` p50 went up even though `get` code was untouched. Each access makes four scattered counter writes, and that is where the `get` time goes.
- Sketch-input levers are used up from both directions. More width buys nothing measurable. Filtering first sightings (the doorkeeper) hurt, because a small filter fills with one-hit keys and its all-at-once clear acts as abrupt aging. That matches the earlier halving and aging failures. The strict frequency duel with non-decaying counters still wins on workload-a.
- Refactors that preserve behaviour still keep hit rate exactly (v001), so the CLOCK base stays safe to change. The p99 readings still move in both directions with no explanation in the diffs.

## Open questions

- Would a smaller or denser sketch (narrower counters, rows packed into one cache line per key, or fewer rows) cut `get` latency without losing hit rate? This is the reverse of this cycle's widening.
- Is workload-a's hit rate at its ceiling for strict-duel TinyLFU on CLOCK? Every admission, aging, window and sketch-accuracy change since the early cycles has been flat or negative.

## Continuation

Keep cycle 0006's v001 (CLOCK, width factor 8) as the base, and adopt none of this cycle's variants. Drop the following:
- hash-once and identity-keyed maps
- further sketch widening
- doorkeepers and any periodic or abrupt aging

If latency is the target, cut the sketch's memory traffic on each access (for example, rows blocked into one cache line, or a smaller footprint). Require hit rate to stay identical or within noise, and judge by medians repeated side by side against the parent. Otherwise, treat hit rate as near its ceiling and prefer behaviour-preserving changes.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
