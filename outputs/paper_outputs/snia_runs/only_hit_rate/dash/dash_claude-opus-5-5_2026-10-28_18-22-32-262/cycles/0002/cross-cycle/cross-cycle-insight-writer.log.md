# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0002/cross-cycle
- allow_edits: True
- started: 2026-10-28T18:36:47+00:00

## timeline

### 18:37:01 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0002/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Changing where items are placed in the stash has helped more than adding filters. Putting demoted items at the stash MRU end (Winner in cycle 0001, and the base of cycle 0002) raised hit-rate on every trace and size, and it still holds as the base.\n- The two-hit exit gate was the best add-on in both cycles. It won cycle 0002, built on top of MRU demotion. Its gains are steady at the small cache size, where the target tier is crowded and protecting reused items matters most.\n- Gains come mostly from cells where reuse comes soon or contention is high: the small size, and `hm_0` at the large size. Levers that protect reused items from one-hit arrivals pay off there.\n\n## Recurring failures\n\n- Every extra stash filter has cost hit-rate on `web_0` at the large size: the fixed gate (both cycles), frequency comparison, and LRU-end insertion. When target buckets have spare room, plain first-hit promotion is already close to optimal. No variant has recovered this cell yet.\n- Stash levers interact instead of adding up. They all act on the same tiny bucket-local population, so one can undo another. For example, resetting counters on demotion cancels the benefit of MRU-end demotion.\n- Frequency signals are too weak in this design. Counters reset or halve on moves, and promotions land at the target's eviction end, so the TinyLFU-style comparison ended up acting like first-hit promotion.\n- Inserting fresh puts at the stash LRU end takes away new keys' probation hit. In effect it means \"evict on next collision\".\n- per-op-time and metadata-size have not been reported in any cycle. Every cost claim (lookup reorder, cheaper gate) is unverified, and cost-only control variants have given no usable signal.\n\n## Open questions\n\n- Does applying the two-hit gate only to fresh puts remove the `web_0` large-size regression? In that version, demoted items keep their count or promote on their first stash hit.\n- Would a gate that scales with occupancy or capacity keep the small-size gains while dropping the gate in roomy caches?\n- Would frequency admission work if the victim were the bucket's lowest-counter item instead of LRU index 0, and counters survived moves?\n- Can a softer fresh-key insertion (one slot above LRU, or a per-bucket bimodal choice) keep the `hm_0` large-size gain without starving new keys?\n- Does the target-first lookup reorder lower get latency? Its hit-rate is identical to baseline, so it is risk-free to add, but it is unproven as a cost win until per-op-time is measured.\n\n## Per-policy notes\n\n- Stash/target two-tier design: what goes wrong is the stash ordering, i.e. reused items getting evicted before fresh or never-hit ones. Fixes to entry position and exit criteria have been the most productive levers.\n- LRU-within-bucket stash: the slots are tiny, so position changes have large effects. LRU-end placement is almost the same as immediate eviction.\n- Frequency/TinyLFU-style admission: it has not worked so far because the counters never build up. Fix counter persistence and victim choice before trying it again.\n- Promotion gating: it shows a size-dependent tradeoff (helps the small size, hurts the roomy large size on `web_0`). That makes it a candidate for adaptive or occupancy-aware thresholds.\n"
}
```

### 18:37:07 chat

I've written `cross-cycle-insights.md` in the cycle 0002 cross-cycle folder, using the four required sections in order. It draws only on the two ledger rows (cycles 0001 and 0002).

- **Recurring wins:** putting demoted items at the stash MRU end is still the best base. The two-hit exit gate is the best add-on so far, with gains mostly at the small cache size and on `hm_0` at the large size.
- **Recurring failures:**
  - Every extra stash filter has cost hit-rate on `web_0` at the large size, and no variant has recovered that cell yet.
  - The stash changes interfere with each other instead of adding up.
  - Frequency counters stay too weak to make TinyLFU-style comparison useful.
  - Inserting fresh puts at the stash's least-recently-used end means new keys get evicted on the next collision.
  - per-op-time and metadata-size have never been reported, so every cost claim is unverified.
- **Open questions:**
  - Would applying the gate only to fresh puts remove the `web_0` large-size regression?
  - Would a gate that scales with occupancy keep the small-size gains?
  - Would frequency admission work with a better choice of victim and counters that survive moves?
  - Can fresh-key insertion be made gentler?
  - Does the lookup reorder actually lower latency?
- **Per-policy notes:** these are grouped by the two-tier stash/target design, the least-recently-used ordering inside each bucket's stash, frequency admission, and promotion gating. Gating looks like a good candidate for a threshold that adapts to how full the cache is.


---

- finished: 2026-10-28T18:37:08+00:00
- duration: 20.65s
- status: ok
- input_tokens: 4
- output_tokens: 1786
- cache_read_tokens: 35948
- cache_write_tokens: 7262
- cost_usd: 0.101022
