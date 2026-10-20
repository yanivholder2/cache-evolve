# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-10-20T10:40:17+00:00

## timeline

### 10:40:24 thinking



### 10:40:31 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-cycle insights\n\n## Recurring wins\n\n- Per-op time is the most reliable lever. Constant-factor changes that leave admission and eviction order alone (shared hashing, hoisted seeds, shared unlink/promotion paths) recur as speed gains with hit rate flat (Cycles 0001, 0002, 0004, 0005).\n- Seed and bookkeeping storage belongs in shared constant tables, not per-instance fields. Moving seeds to a module-level table removed the struct growth that per-instance caching had caused (Cycles 0004, 0005, 0006).\n- Metadata can shrink independently of policy. Packing protected state into existing fields cut size without moving hit rate (Cycle 0002).\n- Collision spread is the one lever that has moved hit rate upward recently. A conservative sketch update and a wider-but-shallower sketch both gave small gains, and a narrower-counter, wider-sketch layout did too (Cycles 0008, 0009). Each stayed within the baseline memory budget.\n\n## Recurring failures\n\n- Changes to admission or aging have not raised hit rate on this workload. Recency rotation of rejected victims, a doorkeeper, a W-TinyLFU-style window, shorter sketch aging, and admission tie-breaking all left hit rate flat or lower (Cycles 0001, 0002, 0003, 0004, 0007). Stale counts and one-hit pollution do not appear to be the binding constraint.\n- Capping or shrinking the protected segment costs hit rate. Bounding the protected set and halving its share both lost hit rate and per-op time, so capacity is the wrong lever (Cycles 0003, 0007). Protected-tail demotion alone produced no change (Cycle 0006).\n- Layouts that change which keys collide trade hit rate for speed. Cache-line grouping gained per-op time only by changing collisions (Cycle 0006), and taking the column from the per-row seed lost the speed without restoring spread (Cycle 0007).\n- Restoring public per-row indexing costs speed. It recovers admission fidelity but adds per-row work on get and put (Cycles 0003, 0005).\n- Timing claims from single runs have not held. Put and get shifts appeared in changes that should not touch those paths, and several reported gains or regressions could not be tied to a mechanism (Cycles 0004, 0005, 0006, 0008, 0009).\n\n## Open questions\n\n- Generalization is untested. Every cycle used one trace (workload-a) and one cache size (2048). No gain has been checked on a second trace or size.\n- Can collision-spread gains be kept without an extra pass over all rows? This applies to the conservative update and the row/column trade.\n- Do narrow saturating 8-bit counters cost admission accuracy? The hit-rate gain was small and the saturation effect was not isolated.\n- Do the seed-table indices match per-access derivation for every key, or only in aggregate hit rate?\n- Do the put-side tail gain and median regression in the single-loop conservative update survive repeat runs?\n- Is the doorkeeper's and the window's memory cost reported at all? Neither storage shows in the metadata breakdown.\n- What caused the unexplained metadata movements: the struct growth in one variant, and the smaller sketch size in another?\n\n## Per-policy notes\n\n- TinyLFU (frequency-sketch admission gate): the sketch is where hit-rate movement has come from. Collision-reducing updates and layouts help slightly; admission-side bookkeeping (doorkeeper, tie-breaks, windows, aging) has not. Counter width and row/column shape are the open tuning axes.\n- Segmented LRU (probation/protected lists): protected-segment capacity, not turnover, drives the trade-off. Shrinking the protected share or capping it lost hit rate. Pointer-work trims on the MRU path are safe speed wins.\n- Trace coverage: all policy-level evidence is from one trace (workload-a) at one size, so no per-policy result is yet general.\n"
}
```

### 10:40:34 chat

I wrote `cross-cycle-insights.md` to the cycle 0009 cross-cycle output directory, using the four required sections in order. I built it only from the inlined ledger digest and did not open the previous file.

The main points across cycles 0001–0009:

- **Recurring wins:** Per-op time gains from constant-factor work that leaves eviction and admission order alone. Seed tables kept in shared constants rather than per-instance fields. Packing state into existing fields to cut metadata. Collision-reducing sketch updates and layouts are the only recent source of hit-rate gains.
- **Recurring failures:** Admission and aging changes (doorkeeper, window, tie-breaks, shorter aging, recency rotation) have not raised hit rate. Shrinking or capping the protected segment costs hit rate. Layouts that change collisions trade hit rate for speed. Single-run timing claims have not held up.
- **Open questions:** Every result comes from one trace (workload-a) at one size, so nothing is shown to generalize. Other open items include whether collision gains can avoid the extra per-row pass, whether 8-bit saturating counters hurt admission, and whether seed-table indices match the per-access derivation for every key.
- **Per-policy notes:** Grouped under TinyLFU (sketch), segmented LRU (protected/probation), and a trace-coverage caveat.

The digest does not name a second policy beyond TinyLFU and the segmented list, so the per-policy section is thin.


---

- finished: 2026-10-20T10:40:34+00:00
- duration: 17.05s
- status: ok
- input_tokens: 4
- output_tokens: 3161
- cache_read_tokens: 40650
- cache_write_tokens: 13866
- cost_usd: 0.004761
