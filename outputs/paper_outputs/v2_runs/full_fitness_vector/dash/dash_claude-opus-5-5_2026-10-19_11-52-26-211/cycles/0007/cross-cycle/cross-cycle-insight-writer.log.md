# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-10-19T14:12:26+00:00

## timeline

### 14:12:45 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **Treating the stash as a probation queue ordered by reuse (c1–c2).** Demotees go to the stash's MRU end and fresh puts go to its LRU end. The two gains add up. This is still the base placement for every later lineage.\n- **Using idle capacity is the only lever that has kept raising hit-rate (c4–c6).** Two sources worked: the two-choice neighbour stash, and rescuing stash victims into holes in their own target or probing bucket. They draw on different holes, so their gains stack. Neither adds tracked metadata.\n- **Filling holes late, not early.** Rescuing items at stash eviction time pays off. Filling holes at insert time crowds out promotions that have proven reuse (c5).\n- **Lossless gates are the only reliable way to cut get latency (c6–c7).** Exact per-segment counters skip neighbour scans without changing cache state: `borrowed` gates the `s1` scan and `foreign` gates the probing scan. The gates stack, so a full miss can skip both neighbour probes. A counter costs nothing when each mutation already knows the item's class; it costs `put` time when it has to re-hash.\n- **Reordering lookups without changing state.** A key lives in at most one slot, so probe order changes only latency, never hit-rate. Probing `s1` last moves the extra scan cost onto misses only (c5). Exact hit-rate matches have confirmed this every time.\n- **Metadata-size has never moved.** Every lever so far gets its extra choice from the hash or from bucket fullness.\n\n## Recurring failures\n\n- **Admission filters on either promotion edge (c2–c3).** Three were tried: MRU placement on stash→target, a second-hit gate, and a native-ownership gate on target→probing. Each one blocked real reuse. The buckets are too small for any extra wait to pay off.\n- **Fixed lookup-order changes as a lasting win.** The best order flips whenever placement changes the tier mix (c1 vs c3). Reordering also can't remove scan cost that comes from fuller buckets (c6).\n- **Bookkeeping or hash trimming to reach p50 or to fix rescue's cost (c4, c7).** The median is set by scans and by promotion moves. Rescue's cost is structural: fuller buckets mean more cascades.\n- **Second sources of idle capacity that are close to empty.** Probing holes are rare at native-demotion time (c6). An independently hashed `s1` adds little because stash overflow is barely clustered, and the extra hash slows every op (c7). Hot-path branches that rarely fire cost latency for no gain.\n- **Per-op timings without a code-level mechanism.** Several cycles showed environment-driven shifts and unexplained tail moves (c1, c7).\n\n## Open questions\n\n- Does rescue stacked on the dual-gate parent keep the best hit-rate? Do the gates' savings cancel rescue's cost from scanning fuller buckets, or does `foreign` stop being mostly zero once rescue writes into probing?\n- Can a cheaper rescue keep most of its hit-rate while cutting cascades and tail cost? Candidates: target-only, scan-free (only when a slot is known free), or rate-limited.\n- Can `borrowed` be fed from the caller's `s0` or native flag so it never re-hashes, and does put p99 recover?\n- Is c7's unexplained drop in put p99 real? Re-measure it alongside the baseline.\n- Can in-bucket tags or fingerprints shorten scans in fuller buckets at an acceptable metadata cost?\n- Can the promotion cascade be made cheaper without changing what gets promoted, e.g. in-place swaps or a bounded cascade depth?\n- Stash capacity and associativity, and eviction order inside target buckets, remain untested structural levers.\n\n## Per-policy notes\n\n- **Stash:** Fresh items at Front and demotees at Back form a scan-resistant queue. Two-choice placement with `s0+1` adds hit-rate. An independent second hash is not worth its cost. The `borrowed` gate makes `s1` nearly free on misses, but it still re-hashes on removal.\n- **Target buckets:** Front placement on promotion acts as a useful admission filter; MRU placement hurts. Rescue into target holes pays off. Fresh-insert fills into target holes push cost onto later promotions.\n- **Probing buckets:** Gating admission by ownership hurts hit-rate. Holes at native-demotion time are almost nonexistent. The `foreign` counter is free to maintain and cleanly cuts get p50/p99.\n- **Lookup path:** Hit-rate and get latency now trade off directly through residency: more items resident means more scanning per lookup. The way to cut latency is to skip scans losslessly, not to reorder them or trim bookkeeping.\n"
}
```

### 14:12:52 chat

I wrote `cross-cycle-insights.md` in the cycle 0007 cross-cycle directory, using the four required sections. It is built only from the ledger for cycles 1–7.

- **Recurring wins:** Three things keep paying off.
  - Using the stash as a probation queue: fresh puts go to the low-priority end and demoted items go to the high-priority end.
  - Using idle capacity raises hit-rate. Both the neighbour stash bucket and rescuing evicted stash items into free normal-bucket slots work, and the two gains add up.
  - Exact counters that skip scans without changing what the cache holds cut get latency, and they stack. Metadata-size has not moved in any cycle.
- **Recurring failures:** These have not paid off.
  - Stricter rules for when items get promoted, on either promotion path.
  - Fixed lookup-order changes: the best order flipped once placement changed.
  - Trimming hashing or bookkeeping.
  - Sources of spare capacity that are nearly empty, such as an independently hashed second stash bucket.
  - Trusting latency changes that nothing in the code explains.
- **Open questions:** The main threads are:
  - Whether rescue stacks on the parent that uses both counters, and whether a cheaper form of rescue keeps most of its hit-rate.
  - Keeping one of the counters up to date without re-hashing.
  - Re-measuring cycle 7's unexplained drop in put tail latency.
  - Small per-slot tags to shorten scans, and cheaper promotion chains.
  - Stash size, which has never been tested.
- **Per-policy notes:** Grouped by stash, target buckets, probing buckets and the lookup path.


---

- finished: 2026-10-19T14:12:52+00:00
- duration: 26.40s
- status: ok
- input_tokens: 4
- output_tokens: 2320
- cache_read_tokens: 39772
- cache_write_tokens: 12853
- cost_usd: 0.157194
