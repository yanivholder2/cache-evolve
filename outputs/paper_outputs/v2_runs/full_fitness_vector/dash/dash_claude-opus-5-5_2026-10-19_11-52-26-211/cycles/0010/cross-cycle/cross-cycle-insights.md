# Cross-Cycle Insights (cycles 0001–0010)

## Recurring wins

- **Lossless, state-neutral changes are the only reliable way to cut get latency.** Lookup reorders (c1, c5), exact per-segment occupancy gates (`borrowed` c6, `foreign` c7), single-hash plumbing (c8) and inline key tags (c9) all kept hit-rate exactly the same while lowering latency. These gates stack. Exact hit-rate matches give a cheap check that a change really is state-neutral.
- **A counter costs nothing only when its inputs are already known.** `foreign` was free because slot type and `true_target` were already at hand. `borrowed` cost put time because it re-hashed keys. When adding bookkeeping, feed it values the caller already computed.
- **Avoid the slow load instead of restructuring around it.** After hashing and dispatch were ruled out, the get-median floor turned out to be the dependent load into each bucket's heap item buffer (c8–c9). Tags, which skip that load on probes that miss, beat flattening the layout, because tags left the mutation paths alone.
- **New places for keys to land raise hit-rate; admission tuning does not.** Hit-rate rose with the stash ranked by proven reuse (demotees at Back c1, fresh puts at Front c2), the two-choice stash (c4–c5) and stash-victim rescue into target holes (c4–c6, c8). None of these added metadata.
- **Target-only rescue keeps almost all of full rescue's hit-rate.** Probing holes add almost nothing, and keeping rescue out of probing keeps the `foreign` gate clean (c6, c8).
- **Metadata-size has never moved.** Every lever so far gets its extra placement choice from the hash or from how full a bucket is.

## Recurring failures

- **Admission filters on either promotion edge.** Second-hit gates, ownership gates and MRU placement on stash→target promotion all blocked real reuse (c2, c3). The buckets are too small for an extra wait to pay off.
- **Rescue in any plain form costs get latency.** Its cost comes from cascades, not scanning: fuller targets send promotions into full buckets and set off demote→stash→rescue chains. This happened in c6, c7, c8 and c10, and tags did not remove it.
- **Bookkeeping and hash trimming as a route to get p50.** These changes only reach the tail, because the promote/demote chain is a tail event (c4, c7, c8).
- **Devirtualisation and layout changes that keep the heap hop.** A concrete bucket over a heap `Vec` (c8) moved nothing. Shrinking the header (c10) moved nothing because the hot fields already fit in one cache line.
- **Recency stored as position in inline/flat layouts.** Every insert, remove and reorder becomes a shift, which pushes up put p50/p99 (c9).
- **Stash geometry and the idle-capacity supply.** Independent-hash `s1` (c7), wider pooled stash buckets (c10) and native-demotion into probing holes (c6) all gained little or lost hit-rate. Stash overflow is not clustered, so this direction is closed.
- **Eager filling of holes.** Putting fresh items straight into target holes (c5) fills targets with cold items and turns later promotions into cascades.

## Open questions

- **Stash insertion asymmetry.** Fresh overflow keys go in at the LRU end while cold demotees go in at the MRU end. Would flipping either one raise hit-rate for free? It changes no state shape and adds no scans.
- **Rescue with a cascade guard.** Could rescue run only below a target fill threshold, or only when a promotion hole is kept free, or only for victims that have shown reuse? The aim is to keep the hit-rate without the cascade cost.
- **Inline storage plus recency that doesn't move items.** Could per-slot age/clock bytes or an index permutation be combined with tags, and would they stack?
- **64 B alignment of the bucket array.** This is a cheap side experiment and still untested.
- **Can the promotion cascade itself be made cheaper without changing which items get promoted?** Options include in-place swaps or limiting how deep a cascade goes.
- **Re-measure the put-p99 baseline.** Put-tail swings without a code mechanism keep showing up (c7–c10). Don't credit write-path claims until this is done.

## Per-policy notes

- **Current latency parent:** the c9 v002 tagged-header lineage. Its stack is two-choice stash with `s1` probed last, the `borrowed` and `foreign` gates, single-hash plumbing and inline tags. Nothing in c10 beat it.
- **Stash (LRU, probationary queue):** recency ordering inside the stash is the main hit-rate lever, since the stash holds most of the capacity. Stash-first lookup became better once fresh-at-Front sent first reuses through the stash. The best lookup order depends on placement policy, so re-check it whenever placement changes.
- **Target/probing buckets:** the plain LRU-ordered promotion with Front placement acts as a useful admission filter, and every stricter or hotter alternative lost. Fuller target buckets reliably trade get latency for hit-rate through cascades.
- **Latency attribution:** credit per-op-time changes only when the code gives a mechanism for them. Environment shifts (c1) and put-p99 variance have repeatedly produced gains or losses that weren't real.
