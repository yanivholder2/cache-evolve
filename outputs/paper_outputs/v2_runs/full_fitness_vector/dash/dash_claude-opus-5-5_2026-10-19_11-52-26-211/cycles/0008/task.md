# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008

## Winners

- v003
- v001
- v002

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/hypotheses/H03.md

## Prior cycles digest

### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three variants branched from cycle-4 v001 and used spare capacity in different ways. v001 added the two-choice stash but probes `s1` last. v002 combined the two-choice stash with normal-tier victim rescue. v003 puts fresh items into free target slots at insert time. All three raised hit-rate over the parent, and v002 confirmed that cycle 4's two levers stack. Only v001 met its latency claim, and only relative to cycle-4 v002. Every variant is still slower than the parent on get, and v003's latency claim was refuted.

## Patterns

- Using idle capacity is still the only lever that raises hit-rate. Every variant that keeps items in neighbour stash buckets or normal-bucket holes gains. Metadata-size stayed flat in all of them. The neighbour-stash and rescue levers draw on different capacity, so their gains add up. v002 can't tell how much each one contributes.
- The cost of extra residency shows up in latency, and the lookup order decides who pays it. Scanning `s1` before the normal tiers charges every target and probing hit. Scanning it last (v001) charges only misses and the rare `s1` hits, with no change in cache state. The reorder is state-neutral because a key lives in at most one slot, and v001's exact hit-rate match confirms this. The same argument should carry over to the rescue structure.
- When an item fills a hole matters. Filling holes late (rescuing stash victims) costs nothing on lookup. Filling them early (fresh puts into target) fills the normal tier with cold items. Later reuse-proven promotions then pay for eviction and demotion cascades that used to be free inserts. The cost moves to promotions instead of going away.
- The get-latency gap that remains on the two-choice structure comes from misses scanning one more bucket. Further reordering can't close it, because `s1` is already last.

## Open questions

- Does v001's `s0 → target → probing → s1` order on top of v002's two-choice + rescue placement keep v002's hit-rate exactly and recover its get-p50 cost? It should, but this hasn't been verified.
- Can most `s1` probes on misses be skipped cheaply? One option is a per-`s0` "has overflowed" bit, another is a fingerprint. What does that do to metadata-size?
- Does fresh-into-target-hole add anything on top of rescue, given they compete for the same holes? Could fresh items be placed so they always give way to promoted items, without competing at the target's LRU end?

## Continuation

Make v002's placement (two-choice stash + rescue) the new parent, combined with v001's lookup order that probes `s1` last. Check that hit-rate is unchanged and get p50 improves. Next, go after the remaining get cost on misses with a cheap `s1` skip filter, and judge any metadata cost against the latency gain. Set aside v003's eager fresh-into-target fill: it trades latency for hit-rate in a way rescue already covers more cheaply. Revisit it only in a form that never blocks later promotions.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three variants branched from cycle-5 v001 (two-choice stash, `s1` probed last). v001 ported stash-victim rescue. v002 added an exact per-stash-bucket borrowed counter that lets `get` skip the `s1` scan. v003 placed native target victims into holes in their probing bucket. Only v002 confirmed its hypothesis: it cut get p50 and left hit-rate exactly unchanged. v001 matched cycle-5 v002's hit-rate, but its latency claim was mostly refuted. v003's hit-rate gain was within noise.

## Patterns

- Cycle 5 blamed v002's get cost on lookup order. Cache state is the real cause. Rescue keeps normal buckets fuller, so every target/probing hit and every miss scans more occupied slots, whatever the probe order. The hit-rate vs get-latency trade from extra residency is now a cost per scan, and reordering cannot remove it.
- Lossless filters work. A small exact counter, stored per segment and not per item, skips scans with no state change and no growth in tracked metadata. Its cost lands on `put`, and mostly in the tail, because the bookkeeping re-hashes keys the insert/evict path already hashed.
- The supply of idle capacity is running out. Stash-victim rescue still pays off. Probing holes at native-demotion time almost never exist in steady state, so that source can be ruled out. Adding branches on hot paths that rarely fire costs a little latency and gains nothing.
- Every variant raised `put` cost in some form: extra rescue checks, re-hashing for the counter, or longer cascades into fuller buckets. Write-path cost is now an axis to track, not noise.

## Open questions

- Does v002's borrowed-counter gate stack with v001's rescue structure? Does it still skip most `s1` probes once rescue changes stash turnover?
- Can the counter be maintained without re-hashing, by passing `s0` or a native/borrowed flag from the caller, so that put p99 comes back?
- Can in-bucket filtering (tags/fingerprints) shorten the longer scans that rescue causes, and what would it cost in metadata?

## Continuation

New parent: v001 (rescue + `s1`-last) with v002's borrowed-counter gate, with the counter fed the already-computed `s0` / native flag so no re-hashing happens. Check that hit-rate holds and that get p50 and put p99 both improve. Then target the scan cost of fuller normal buckets with in-bucket filtering, weighing it against metadata-size. Drop native-demotion-into-probing-hole and further probe-order changes.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

v001 and v002 branched from cycle-6 v002 (the counter-gated lineage). v003 branched from cycle-6 v001 (the rescue lineage). v001 merged rescue with the `borrowed` gate and shared one hash between them. It matched the best hit-rate exactly, but its put tail claim was refuted and its get tail grew. v002 added an exact `foreign` counter per normal bucket to gate the probing scan. It cleanly confirmed: get p50 and p99 both dropped, and hit-rate held. v003 replaced the `s0+1` stride with an independently hashed `s1`. It gained a little hit-rate, but the extra hash made every per-op metric worse.

## Patterns

- Exact occupancy counters only pay off when every mutation already knows how to classify the item. `foreign` cost nothing on put because the slot type and the demotion's `true_target` already carry that information. `borrowed` still rehashes in its removal path and shows a put cost. A counter is free when its inputs are free.
- Lossless gates stack. v002 runs the `s1` and probing gates together, so a full miss can now skip both neighbour probes. With no change to cache state, this is still the only reliable way to cut get latency.
- Rescue's cost is structural, not hashing. Sharing one hash fixed bookkeeping, but rescue still does extra normal-bucket work on every stash overflow. It also leaves normal buckets fuller, so promotions inside `get` evict and cascade more often. That pushes up both get and put tails. More hash trimming won't recover this.
- Stash overflow is barely clustered. Independent second choice adds little hit-rate, so the idle-capacity lever is close to exhausted on the stash side. Any extra full hash on a hot path costs more than it gains.

## Open questions

- Is v002's sharp put-p99 drop real? Nothing in the change explains it, so it needs re-measurement before anyone builds on it.
- Do the `foreign` gate's get savings cancel rescue's fuller-bucket scan cost when combined? Does `foreign` stay mostly zero once rescue writes into probing slots?
- Can rescue be trimmed (target-only, or rate-limited) to keep most of its hit-rate while dropping the cascade and tail cost?

## Continuation

Make v002 (both gates) the latency parent. Next, stack rescue onto it in a cheaper form: target-only, or only when the target slot is free without a scan. Also feed `borrowed` from caller-known `s0`/native flags so it never rehashes. Re-measure put p99 before trusting it. Drop independent-hash `s1` unless it is derived from spare primary-hash bits and computed only after `s0` is full. Drop further hash-trimming as a fix for rescue.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
