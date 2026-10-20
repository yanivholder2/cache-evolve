# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-10-20T14:59:31+00:00

## timeline

### 14:59:37 thinking



### 14:59:43 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-cycle insights\n\n## Recurring wins\n\n- Gating stash promotion on re-reference is the one lever that has repeatedly shown a hit-rate signal. Requiring a second hit (cycle 2) and stacking that gate on MRU landing (cycle 3) both helped, though only on the single trace.\n- Landing a re-referenced stash item at the MRU end of its target bucket is the original hit-rate gain (cycle 1). It has been carried forward unchanged since, and no later cycle has improved on it.\n- Probing-tier residency is the binding resource. Shrinking the probing tier to grow the stash lost hit-rate (cycle 8), and reversing that at fixed total capacity gained it (cycle 9). The two results agree in direction.\n\n## Recurring failures\n\n- Reuse-counter refinements have produced no robust signal for four cycles (6, 7, 8, 9). Increment rules, reset-on-demotion, saturation gates, and collapsing the counter back to a one-bit marker all landed at parent level or within noise.\n- Using the seen-in-stash signal as an eviction preference hurt (cycle 3). It discarded recently arrived items that had not yet had a chance to prove reuse.\n- Early readmission of recently evicted keys, via a ghost-history path that skips the wait, cost a little hit-rate (cycle 5).\n- Movement-adding or retry-removing changes regressed. Remove-and-reinsert on probing hits, direct-to-stash eviction, and skipping the target retry for foreign evictions all lost hit-rate (cycle 1). A capacity-gated variant of the shortcut recovered only part of that loss (cycle 2).\n- Stash-victim ordering and occupancy-conditional landing or gating tweaks have been flat (cycles 2, 4). Eviction-order and landing refinements in this part of the design have shown no effect so far.\n- Behaviour-preserving refactors (occupancy counters, skipped slot scans, store-skip, gate reorder) match their parent by construction. Their claimed speed benefit has never been measured, so they have not shown value.\n\n## Open questions\n\n- Do the winning levers (re-reference promotion gate, MRU stash landing) hold on workloads and capacities other than LIRS workload-a at size 2048? No cycle has tested a second trace.\n- Where is the probing-tier optimum? The gain from growing probing may flatten or reverse beyond 128 slots.\n- Does per-op-time or metadata-size move under any change? Neither has been recorded in any of the nine cycles, so the cost side of every variant is unknown, including the bit cost of any reuse counter or marker.\n- Several cycles are labelled Winner with hit-rate at or below the parent (cycles 4, 5, 7, 8). Which baseline these wins are measured against is unclear, so the winner labels should not be read as evidence of improvement.\n- Most insights describe intended changes rather than verified overlay source. Whether the implementations match their descriptions is unconfirmed.\n\n## Per-policy notes\n\n- All nine cycles evaluate one policy family: a tiered design with probing, stash, and normal tiers, judged by hit-rate on LIRS workload-a at size 2048. The ledger has no per-policy split, so the notes below apply to that family only.\n- Admission and promotion (probing → stash → normal): re-reference gating is the most promising direction. Probing-tier admission filters and back-to-back filters were flat.\n- Landing and placement: MRU landing for stash hits is established. Demoted-item placement at the LRU end was flat (cycle 5).\n- Eviction and victim selection: stash victim preferences (never-re-hit, earliest-landed) were harmful or flat. Foreign-eviction retry removal regressed.\n- Reuse tracking (counter or marker): no robust signal across cycles 6–9. Treat as low priority unless a second workload shows a gain.\n"
}
```

### 14:59:45 chat

I wrote `cross-cycle-insights.md` to the cycle 0009 cross-cycle directory, with the four required sections in order. I built it only from the inlined ledger digest and did not read any previous version of the file.

The main points across the nine cycles:

- **Recurring wins:** Gating stash promotion on a second re-reference is the one lever that has shown a hit-rate signal more than once. MRU landing for stash hits from cycle 1 has held up unchanged. Probing-tier residency is a binding resource: shrinking the probing tier lost hit-rate in cycle 8, and growing it back gained in cycle 9.
- **Recurring failures:** Reuse-counter refinements have been flat or within noise for four cycles. Using the seen-in-stash signal as an eviction preference hurt. Retry-removing and movement-adding changes regressed, and the refactors that match their parent have never had their claimed speed benefit measured.
- **Open questions:** Every result comes from one trace (LIRS workload-a at size 2048), so none of the winners has been tested on a second workload. Per-op-time and metadata-size have never been recorded. Several cycles are labelled Winner while their hit-rate is at or below the parent, so the winner labels are not reliable evidence of improvement. Most insights also describe intended changes rather than verified overlay source.
- **Per-policy notes:** The ledger covers one policy family (the probing/stash/normal tiered design), so there is no real per-policy split. The notes are grouped by mechanism within that family.

I did not commit the file, since you asked only for the write.


---

- finished: 2026-10-20T14:59:46+00:00
- duration: 15.52s
- status: ok
- input_tokens: 4
- output_tokens: 3041
- cache_read_tokens: 40355
- cache_write_tokens: 13416
- cost_usd: 0.004608
