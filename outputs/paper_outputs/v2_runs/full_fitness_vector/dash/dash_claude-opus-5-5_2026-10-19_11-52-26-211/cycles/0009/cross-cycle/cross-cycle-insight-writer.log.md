# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-10-20T09:32:17+00:00

## timeline

### 09:32:38 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (through cycle 0009)\n\n## Recurring wins\n\n- **Using idle capacity is the only hit-rate lever that has kept working (cycles 2, 4–8).** Gains came from stash order that ranks proven reuse (demotees at stash Back, fresh puts at stash Front), the two-choice neighbour stash, and rescuing stash victims into holes. None of these added metadata. Rescue that writes only into target holes keeps almost all of full rescue's gain.\n- **State-neutral, lossless filters reliably cut get latency (cycles 6, 7, 9).** Examples are the exact per-segment `borrowed`/`foreign` counters that gate the `s1` and probing scans, and the inline one-byte key tags. Hit-rate stayed exactly the same each time. The gates stack, so a full miss can skip both neighbour probes.\n- **Gates are cheapest when mutations already know the item's class (cycle 7).** Counters fed by slot type or a hash the caller already computed cost nothing on put. A counter that has to rehash shows up in the put tail.\n- **Avoiding the dependent load into each bucket's heap item buffer finally moved get p50 (cycle 9).** Tags beat flattening because they skip the load on missing probes, which are most probes, and they leave mutation paths alone. They also help put, through the \"already present?\" check.\n- **Single-hash plumbing along the stash-hit/demotion chain trims the get tail at no cost (cycle 8).** The chain is a tail event, so the payoff is at p99 only.\n- **Independent changes compose cleanly and predictably.** Examples are lookup order with insert position, two-choice with rescue, and the two gates together. When a refactor claims to be state-neutral, an exact hit-rate match has been a dependable check.\n\n## Recurring failures\n\n- **Admission filters on either promotion edge (cycles 2–3).** Tried: MRU placement on stash→target, a second-hit gate, and a native-ownership gate. Every one blocked real reuse. The tiny buckets can't hold items long enough for an extra wait to pay off.\n- **Fixed lookup-order tweaks as a latency lever (cycles 1, 3, 5, 6).** Which order wins flips with placement policy. Once `s1` is last, the remaining cost comes from cache state (fuller buckets), not from probe order.\n- **Removing hashes or dispatch to move get p50 (cycles 4, 7, 8).** Bookkeeping trims, devirtualising at bucket level, and concrete buckets that still point to a heap `Vec` all left the median flat. Several also made the tails worse.\n- **Recency ordered by position in inline/flat slot layouts (cycle 9).** Every insert, remove or reorder turns into shifting, which raised put p50 and p99.\n- **Low-yield residency sources.** Eagerly filling target holes with fresh items fills the tier with cold items and moves the cost onto later promotions (cycle 5). Probing holes at native-demotion time almost never exist (cycles 6, 8). An independent-hash `s1` adds little hit-rate for the cost of a full extra hash (cycle 7).\n- **Trusting the put-p99 baseline.** It has swung in several cycles (7–9) without a code-level cause. Per-op-time claims without a mechanism have repeatedly turned out to be noise or environment drift (cycle 1 onward).\n\n## Open questions\n\n- Right-sized tags with a smaller `max_size` (a `u8`) and a smaller header stride: how much more get latency comes back?\n- Do tags stack with flat/inline slot storage if recency doesn't move items (age/clock bytes or an index permutation)?\n- With cheaper probes, is target-only rescue's per-resident scan cost now low enough to justify its hit-rate? Can rescue be throttled by victim reuse, or set to give way first?\n- Stash geometry (wider, fewer stash buckets / associativity) is still untested. Cycle 9's attempt produced no result. Could larger stash buckets make a second-hit filter viable?\n- Can `borrowed` be maintained entirely from flags the caller already knows, so it never rehashes? Does `foreign` stay near zero once rescue is stacked on?\n- Re-measure the put-p99 baseline in the same run before crediting any write-path change.\n\n## Per-policy notes\n\n- **Stash (probationary LRU tier):** Main capacity and hit-rate lever. Scan-resistant ordering works: fresh items at Front, demotees at Back. Two-choice overflow helps, but overflow barely clusters, so extra choices add little. Probe `s1` last and gate it with `borrowed`.\n- **Target buckets:** Tiny, so promotion placement at MRU and admission gates both hurt. Holes here are the only rescue source worth using. Fuller target buckets lengthen scans and add promotion cascades, so this is the core hit-rate vs get-latency trade.\n- **Probing buckets:** Keep them clean of rescue so the `foreign` gate stays effective. Native-demotion-into-probing-hole doesn't pay. Gating with `foreign` is free and effective.\n- **Bucket eviction policy implementation:** LRU ordered by position is fine behind a heap `Vec`, but costly when stored inline. Any move to inline storage needs recency metadata that doesn't move items.\n"
}
```

### 09:32:45 chat

I've written `cross-cycle-insights.md` in the cycle 0009 cross-cycle folder. It uses the four required sections in order and is built only from the ledger for cycles 1–9.

- **Recurring wins:**
  - The only thing that has kept raising hit-rate is putting idle capacity to use: stash ordering that favours items already reused, the two-choice stash, and rescuing evicted items into empty target slots.
  - The only reliable way to cut get time has been checks that skip work without changing what the cache holds: the per-segment counters, and the one-byte key tags that finally moved the get median in cycle 9.
- **Recurring failures:**
  - **No gain:** stricter promotion rules on either edge, fixed lookup-order changes, and trimming hashing or dispatch to lower the get median.
  - **Costly:** keeping recency by item position in inline slot storage, because every change shifts items and slows puts.
  - **Low yield:** sources of extra space that rarely pay off.
  - **Untrustworthy baseline:** the put tail (p99) has swung several times with no cause in the code.
- **Open questions:**
  - How much more right-sizing the tag arrays recovers.
  - Whether tags stack with inline slot storage if recency doesn't move items.
  - Whether target-only rescue's hit-rate is worth its cost now that probes are cheaper.
  - Stash size and shape, which is still untested because cycle 9's attempt gave no result.
  - Updating the counter without rehashing.
  - Re-measuring the put-tail baseline before trusting write-path claims.
- **Per-policy notes:** grouped by stash, target buckets, probing buckets, and how each bucket's eviction order is implemented.


---

- finished: 2026-10-20T09:32:46+00:00
- duration: 28.58s
- status: ok
- input_tokens: 4
- output_tokens: 2508
- cache_read_tokens: 41321
- cache_write_tokens: 15842
- cost_usd: 0.185176
