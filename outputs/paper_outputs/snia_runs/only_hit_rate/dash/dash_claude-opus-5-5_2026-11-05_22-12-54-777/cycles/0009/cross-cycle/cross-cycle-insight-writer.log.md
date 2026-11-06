# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-11-05T23:28:02+00:00

## timeline

### 23:28:28 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (cycles 0001–0009)\n\n## Recurring wins\n\n- **Changes to structure beat parameter tuning.** The big steps all changed the structure: the 24/8 topology (c0001), the frequency-aware 8-slot tail victim (c0002), the ghost-credited admission (c0004) and its cache-sized direct-mapped form (c0005), and ghost hits bypassing the stash (c0007). Sweeping a parameter inside one of these structures has usually found the peak within a cycle or two and then stalled.\n- **Changes inside a bucket move the small size.** Victim choice and admission rules have reliably moved the small cache size. Topology-only changes never did.\n- **Ageing works when it targets items already judged cold.** Halve-on-cap, stash-hit halving, and halving on demotion into the stash (c0006) all helped. Ageing that hits recently arrived or Front-landed items has always hurt.\n- **Fixing where proven-reuse keys land beat giving them more credit.** Ghost reach and bonus both stalled until credited keys skipped the short-lived stash (c0007).\n- **Protecting warm residents works better than protecting newcomers.** The contention-gated bypass (c0008) helped small-size hm_0. Exempting newcomers from the victim scan hurt every cell.\n- **A change helps only where the ghost fires.** Every hm_0 gain since c0004, and the only real web_0 gain (large size, c0007), came through the ghost path.\n\n## Recurring failures\n\n- **Insert-position tweaks.** MRU promotion landing hurt web_0 in c0001, c0002 and c0003, alone or bundled. Gated demotion position was redundant once the tail scan reads counters. Front landing is part of how the frequency filter works.\n- **Sweeps that are now used up:**\n  - normal/stash ratio (peaked near 3/4)\n  - tail window width (8 is the peak; both 4 and 16 lost)\n  - counter cap\n  - ghost size above capacity\n  - ghost bonus above 1\n  - fingerprint narrowing (false matches dent web_0)\n  - global gate threshold (it moves the trade-off between sizes but never improves both)\n- **Faster ageing or ageing triggered by eviction count** (c0004, c0007). It drains Front-landed counter-1 arrivals inside the window, and it hurts small caches the most.\n- **Removing ageing points** such as the stash-hit halving (c0005). Items with high counters then stick in target, and every cell gets worse.\n- **Direct scan defence on web_0.** A TinyLFU gate on promotion (c0007) never fired, because one-hit scan keys never reach it.\n- **More placement options under the same cold-only rule.** Neighbour-bucket placement (c0009) failed because the neighbour shares the segment's load.\n- **Learned signals without instrumentation.** The re-hit score (c0009) showed no measurable effect, and nobody could tell whether it rarely fired or was miscalibrated.\n- **Bundling several changes in one variant** (c0002 v001). It hides which change caused a regression.\n\n## Open questions\n\n- Would a bypass gate that depends on contention or occupancy work better than a fixed one: relaxed (≤1) where the target is uncontended or large, strict (==0) where it is contended? Would a tie-break at equal counters that favours the resident help?\n- Large-size hm_0 is close to flat. Is that structural? Do keys leave through paths the ghost never records?\n- How often does each gate branch and ghost path fire? Instrumentation is needed before any adaptive gate is built.\n- Would ageing at other boundaries help, if it targets only cold items: stash→target promotion, or a normal-tier scan that excludes recent promotions?\n- Is there any web_0 lever outside the ghost path?\n- The cost side is still unknown. Per-op-time and metadata-size have not been measured in any of the nine cycles, and overlay landing was unconfirmed for several variants in c0008 and c0009.\n\n## Per-policy notes\n\n- **Current lineage:**\n  1. c0001 v002: 24/8 topology\n  2. c0002 v002: frequency-aware 8-slot tail victim, halve-on-cap\n  3. c0004 v002: ghost ring, admit at counter 1\n  4. c0005 v001: direct-mapped ghost sized to the cache\n  5. c0006 v003: halve on demotion into the stash\n  6. c0007 v002: ghost hits go straight to target Front\n  7. c0008 v002: contention-gated bypass (c0009 v002 is equivalent within noise)\n- **Gains are shrinking.** They have been small since c0006, apart from c0007's structural jump. The current structure looks close to its limit, so a structurally different hypothesis is probably worth a slot.\n- **hm_0 responds most.** It shows the largest gains and the largest regressions. The small-size hm_0 cell is the most sensitive to frequency and admission changes.\n- **web_0 has barely moved.** Its one real gain came at the large size through the ghost bypass. Large-size web_0 tail windows bottom out at counter 1, and that is where the open gap between the two sizes sits.\n- **The best setting depends on cache size.** The bypass/stash and gate-threshold trade-offs flip between the small and large sizes. Future rules likely need to adapt to size or contention.\n"
}
```

### 23:28:34 thinking



### 23:28:34 chat

I've written `cross-cycle-insights.md` with the four required sections, built fresh from the ledger digest for cycles 0001–0009.

- **What kept working:** The big gains came from structural changes, not parameter tuning: the 24/8 topology, the frequency-aware 8-slot tail victim, the ghost table, and sending ghost hits straight to target. Ageing helped only when it hit items already judged cold. Protecting warm residents beat protecting newcomers.
- **What kept failing:** MRU-end landing for promoted items hurt web_0 in three cycles. Every tuning sweep has run out: normal/stash ratio, window width, counter cap, ghost size and bonus, fingerprint width, and the global gate threshold. Faster ageing, removing ageing points, a direct scan gate for web_0, spreading placement to the neighbour bucket, and an untested learned re-hit score all failed too.
- **Open questions:** Would a bypass gate that relaxes or tightens with contention or occupancy work better? Has large-size hm_0 hit a structural limit? The file also asks for counts of how often each gate branch fires, and for any lever that helps web_0. Per-op-time and metadata-size have not been measured in any of the nine cycles.
- **Per-policy notes:** The file traces the parent lineage from cycle 0001 to 0009 and notes that gains are shrinking. hm_0 responds most to changes. web_0 has moved only through the ghost path at the large size. The best setting flips between the small and large cache sizes.


---

- finished: 2026-11-05T23:28:35+00:00
- duration: 33.03s
- status: ok
- input_tokens: 4
- output_tokens: 2453
- cache_read_tokens: 43525
- cache_write_tokens: 15502
- cost_usd: 0.181797
