# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0010

## Winners

- v001
- v003
- v002

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0010/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

All three variants branched from cycle-7 v002, the double-gated latency parent. v001 cut hashing to one per key along the stash-hit/demotion chain, with no change in state. v002 added target-only rescue that needs no scan. v003 replaced `Box<dyn Bucket>` with a concrete bucket type in a contiguous `Vec`. None of the three got the get-p50 gain it was aiming for. v001 lowered the get tail with hit-rate exactly unchanged. v002 recovered almost all of full rescue's hit-rate, but it also paid almost all of rescue's get cost. v003 left the median flat and made the tails worse.

## Patterns

- The median get no longer responds to hashing or to dispatch overhead. Removing hashes and removing the bucket-level vtable hop both left p50 flat. What remains of the median cost is walking resident slots and following the pointer to each bucket's separate heap item buffer. The stash-hit → promote → demote chain is a tail event, so trimming it pays off only at p99.
- Rescue's hit-rate and its latency cost both come from the same source: target holes. Probing holes add almost nothing, and keeping rescue out of probing buckets keeps the `foreign` gate clean. Even so, the cost stays, because fuller target buckets alone lengthen scans and cause more promotion cascades. The hit-rate vs get-latency trade for extra residency in target buckets now looks irreducible as long as the scan cost per resident stays the same.
- Changing data layout only helps if it removes the hop that is actually slow. Swapping a boxed bucket for a header-sized struct that still points to a heap `Vec<Item>` and still holds a boxed policy moved the indirection down one level. The wider bucket stride may also have cost cache lines.
- Put p99 went back up in all three variants. This matches cycle 7's warning that the parent's low put tail was variance. Treat the parent's put-p99 baseline as unreliable.

## Open questions

- Would inline slot storage (fixed `[Item; N]` per bucket, or a flat per-segment item array) together with a static eviction policy finally make each probe cheaper and move get p50?
- If probes get cheaper, does target-only rescue's latency cost shrink enough to make its hit-rate worth keeping? Its cost scales with the number of residents scanned.
- Can rescue be throttled so it limits promotion cascades rather than target occupancy? For example, rescue only when the victim has shown reuse, or place rescued items so they give way first.

## Continuation

Keep v001's single-hash plumbing as the new latency parent: it improves the tail at no cost. Keep target-only rescue as the preferred hit-rate add-on, since it is better than full rescue. Next, make the cost per probe smaller by storing slots inline, with a static policy, in a flat or fixed-array layout. Then re-stack target-only rescue on top of that and re-measure the trade. Stop removing hashes and stop making bucket-level devirtualisation changes while slots stay behind a heap `Vec`: neither will move get p50. Re-measure the put-p99 baseline before judging write-path claims.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

This cycle tested two state-neutral ways to make probes cheaper, plus one hit-rate experiment. v001 (from cycle-7 v002) put each tier's slots into one flat per-segment array and dispatched the policy statically. v002 (from cycle-8 v003) added inline one-byte key tags to the concrete bucket header. Both confirmed their main claim: get p50 moved for the first time in several cycles, and hit-rate stayed exactly the same. v002 was the cleaner win, improving get and put at both median and tail. v001 bought its get-median gain with a clearly worse put path. v003 (wider, fewer stash buckets, from H03) produced no insight, so the stash-geometry question is still untested.

## Patterns

- Cycle 8 was right about the cause: the get-median floor is the dependent load into each bucket's heap item buffer, not hashing and not dispatch at the bucket level. Both winners removed or avoided that load, and both moved p50. The two do it differently. Flattening removes the hop for every probe. Tags skip it for probes that miss, which are most probes, and they help the tail more because full-miss chains touch several buckets.
- Avoiding a load is cheaper than restructuring around it. Tags left the mutation paths almost untouched. Tags also let the "already present?" check before an insert skip the buffer, so put improved too. The flat array made recency ordered by position inside a fixed slice, so every insert, remove, reorder and demotion turned into in-place shifting. That pushed put p50 and especially put p99 up, and nudged get p99 up through move-on-hit.
- Recency ordered by position is now the main mutation cost in any inline layout. Any design that stores slots inline needs recency that doesn't move items: per-slot age/clock bytes or a small index permutation.
- v002's gain is a lower bound. Its tag array is sized for the maximum slot count, not the real capacity, and `max_size` was not shrunk, so the header stride roughly doubled. It improved anyway.
- Put-p99 baselines are still unreliable. One winner's put gain and the other's put loss may both be partly regression toward an earlier level.

## Open questions

- Once tag arrays are sized to real capacity and `max_size` is a `u8`, how much more get latency comes back?
- Do tags and a flat layout stack? For example, tags inline next to a flat slot array, with recency that doesn't move items. Or does the tag filter already capture most of what flattening gives?
- With probes now cheaper, does target-only rescue's per-resident scan cost shrink enough that its hit-rate is worth having?
- Untested: does wider stash associativity (H03) raise hit-rate without rescue cascades?

## Continuation

Make v002 (tagged headers) the new latency parent. First, right-size its tags and header to recover stride. Then re-stack target-only rescue onto it and re-measure the hit-rate vs get-latency trade. Pursue flat/inline storage again only together with recency that doesn't move items, and drop position-shifting recency in fixed slices. Retry the stash-geometry hypothesis, since this cycle produced no result for it, and re-measure the put-p99 baseline before crediting write-path changes.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
