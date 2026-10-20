# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-10-20T09:43:38+00:00

## timeline

### 09:43:58 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (cycles 0001–0010)\n\n## Recurring wins\n\n- **Lossless, state-neutral changes are the only reliable way to cut get latency.** Lookup reorders (c1, c5), exact per-segment occupancy gates (`borrowed` c6, `foreign` c7), single-hash plumbing (c8) and inline key tags (c9) all kept hit-rate exactly the same while lowering latency. These gates stack. Exact hit-rate matches give a cheap check that a change really is state-neutral.\n- **A counter costs nothing only when its inputs are already known.** `foreign` was free because slot type and `true_target` were already at hand. `borrowed` cost put time because it re-hashed keys. When adding bookkeeping, feed it values the caller already computed.\n- **Avoid the slow load instead of restructuring around it.** After hashing and dispatch were ruled out, the get-median floor turned out to be the dependent load into each bucket's heap item buffer (c8–c9). Tags, which skip that load on probes that miss, beat flattening the layout, because tags left the mutation paths alone.\n- **New places for keys to land raise hit-rate; admission tuning does not.** Hit-rate rose with the stash ranked by proven reuse (demotees at Back c1, fresh puts at Front c2), the two-choice stash (c4–c5) and stash-victim rescue into target holes (c4–c6, c8). None of these added metadata.\n- **Target-only rescue keeps almost all of full rescue's hit-rate.** Probing holes add almost nothing, and keeping rescue out of probing keeps the `foreign` gate clean (c6, c8).\n- **Metadata-size has never moved.** Every lever so far gets its extra placement choice from the hash or from how full a bucket is.\n\n## Recurring failures\n\n- **Admission filters on either promotion edge.** Second-hit gates, ownership gates and MRU placement on stash→target promotion all blocked real reuse (c2, c3). The buckets are too small for an extra wait to pay off.\n- **Rescue in any plain form costs get latency.** Its cost comes from cascades, not scanning: fuller targets send promotions into full buckets and set off demote→stash→rescue chains. This happened in c6, c7, c8 and c10, and tags did not remove it.\n- **Bookkeeping and hash trimming as a route to get p50.** These changes only reach the tail, because the promote/demote chain is a tail event (c4, c7, c8).\n- **Devirtualisation and layout changes that keep the heap hop.** A concrete bucket over a heap `Vec` (c8) moved nothing. Shrinking the header (c10) moved nothing because the hot fields already fit in one cache line.\n- **Recency stored as position in inline/flat layouts.** Every insert, remove and reorder becomes a shift, which pushes up put p50/p99 (c9).\n- **Stash geometry and the idle-capacity supply.** Independent-hash `s1` (c7), wider pooled stash buckets (c10) and native-demotion into probing holes (c6) all gained little or lost hit-rate. Stash overflow is not clustered, so this direction is closed.\n- **Eager filling of holes.** Putting fresh items straight into target holes (c5) fills targets with cold items and turns later promotions into cascades.\n\n## Open questions\n\n- **Stash insertion asymmetry.** Fresh overflow keys go in at the LRU end while cold demotees go in at the MRU end. Would flipping either one raise hit-rate for free? It changes no state shape and adds no scans.\n- **Rescue with a cascade guard.** Could rescue run only below a target fill threshold, or only when a promotion hole is kept free, or only for victims that have shown reuse? The aim is to keep the hit-rate without the cascade cost.\n- **Inline storage plus recency that doesn't move items.** Could per-slot age/clock bytes or an index permutation be combined with tags, and would they stack?\n- **64 B alignment of the bucket array.** This is a cheap side experiment and still untested.\n- **Can the promotion cascade itself be made cheaper without changing which items get promoted?** Options include in-place swaps or limiting how deep a cascade goes.\n- **Re-measure the put-p99 baseline.** Put-tail swings without a code mechanism keep showing up (c7–c10). Don't credit write-path claims until this is done.\n\n## Per-policy notes\n\n- **Current latency parent:** the c9 v002 tagged-header lineage. Its stack is two-choice stash with `s1` probed last, the `borrowed` and `foreign` gates, single-hash plumbing and inline tags. Nothing in c10 beat it.\n- **Stash (LRU, probationary queue):** recency ordering inside the stash is the main hit-rate lever, since the stash holds most of the capacity. Stash-first lookup became better once fresh-at-Front sent first reuses through the stash. The best lookup order depends on placement policy, so re-check it whenever placement changes.\n- **Target/probing buckets:** the plain LRU-ordered promotion with Front placement acts as a useful admission filter, and every stricter or hotter alternative lost. Fuller target buckets reliably trade get latency for hit-rate through cascades.\n- **Latency attribution:** credit per-op-time changes only when the code gives a mechanism for them. Environment shifts (c1) and put-p99 variance have repeatedly produced gains or losses that weren't real.\n"
}
```

### 09:44:05 chat

I wrote `cross-cycle-insights.md` from the ledger for cycles 1–10, using the four required sections in order.

The main patterns it records:

- **Lowering get latency:** the only reliable way has been changes that leave cache state alone (exact hit-rate matches confirm this). That covers lookup reorders, the `borrowed` and `foreign` counters that let a lookup skip a bucket scan, single-hash plumbing, and inline key tags. These gates stack. A counter only costs nothing when its inputs are already known, which is why `foreign` was free and `borrowed` cost put time.
- **Raising hit-rate:** this has only happened when keys get new places to land: reuse-ordered stash placement, the two-choice stash, and moving evicted stash items into free slots in their target bucket ("rescue"). Admission filters on either promotion edge have never helped, and metadata-size has never moved.
- **Repeated failures:**
  - Every form of rescue tried so far costs get latency through chains of evictions and demotions, and tags did not remove that cost.
  - Hash and bookkeeping trims only reach the tail.
  - Layout changes that keep the pointer hop to a separate heap item buffer moved nothing.
  - Inline layouts that track recency by slot position make put much slower.
  - Stash geometry is now closed.
- **Next to probe:**
  - Flipping the stash insertion order (fresh keys go in at the LRU end, cold demotees at the MRU end).
  - Rescue only when a guard stops it from triggering eviction chains.
  - Inline storage with recency tracking that doesn't move items.
  - 64 B alignment of the bucket array, as a cheap side test.
  - Re-measuring the put-p99 baseline before trusting any write-path claim.

The per-policy notes name the cycle-9 tagged-header lineage as the current parent; nothing in cycle 10 beat it.


---

- finished: 2026-10-20T09:44:06+00:00
- duration: 28.00s
- status: ok
- input_tokens: 4
- output_tokens: 2617
- cache_read_tokens: 43017
- cache_write_tokens: 16326
- cost_usd: 0.191567
