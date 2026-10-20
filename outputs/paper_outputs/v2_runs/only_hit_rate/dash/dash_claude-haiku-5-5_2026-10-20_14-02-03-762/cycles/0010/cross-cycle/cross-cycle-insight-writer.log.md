# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-10-20T15:04:30+00:00

## timeline

### 15:04:39 thinking



### 15:04:44 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Stash-hit admission and landing rules are the most productive lever. An MRU landing for re-referenced stash items (cycle 0001) and a second-hit gate before stash promotion (cycles 0002–0003) each produced a hit-rate gain, and they stacked modestly.\n- Probing-tier capacity near 128 slots is the best-supported size. Shrinking probing hurt (0008), growing it by 128 slots helped (0009), and growing further to 256 hurt (0010).\n- Keeping proven-hot keys resident in the probing tier matters. Removing and re-inserting probing hits lost ground (0001).\n\n## Recurring failures\n\n- Reuse-counter refinements show no robust signal. Increment rules, reset-on-demotion, back-to-back filters, store-skip, and saturated-counter gates were flat or within noise across cycles 0006–0008, and removing counter states gave only a small drop (0009).\n- Recency-based or victim-preference gates hurt. A close-in-time second-hit promotion gate lost hit-rate (0010), and preferring never-re-hit stash entries as victims discarded items that had not yet had a chance to prove reuse (0003).\n- Shortcutting or removing the demotion/eviction path hurt. Skipping the target retry for foreign evictions and sending probing exits straight to the stash both lost hit-rate (0001, 0002 for the unconditional form).\n- Early readmission of recently evicted keys (ghost-history fast path) gave a slight loss (0005).\n- Pure refactors and placement tweaks match the parent and do not move hit-rate. This covers occupancy counters, gate reordering, marker relocation and folding, earliest-landed victim order, occupancy-conditional landing and gating, and demotion-to-LRU placement (0002, 0004, 0005, 0006, 0010). Their claimed speed or memory benefits were never measured.\n\n## Open questions\n\n- Where is the probing-tier optimum? Is 128 slots the peak, or is there one between 128 and 256?\n- Does the MRU landing and the second-hit gate hold on any workload or capacity other than the LIRS trace at size 2048? No cycle has tested a second workload.\n- What are the per-op-time and metadata-size effects? Neither metric was recorded in any cycle, so every cost trade-off (including the bit cost of the reuse counter and the marker) is unknown.\n- Several Winner designations do not match their measured results: 0005 v001, 0007 v002, 0008 v002, 0010 v003 were listed as Winners with flat or lower hit-rate, and 0002 v001 and 0004 v001 were carried forward without new gain. The baseline used for Winner status should be confirmed before building on these lineages.\n- Several insights describe intended changes, not verified overlay diffs (0002, 0004, 0005, 0006, 0007, 0009). Implementation details, such as marker layout and reset timing, are unconfirmed.\n\n## Per-policy notes\n\n- All cycles evaluate one tiered design (probing, stash, normal) on the LIRS trace at size 2048. Per-policy comparison is not possible from this ledger; no other eviction policy was evaluated.\n- Within this design, the robust signals are about tier sizing and promotion admission. Reuse-counter and eviction-order refinements have not shown a measurable effect.\n"
}
```

### 15:04:46 chat

I wrote `cross-cycle-insights.md` to the output directory with the four required sections in order. I built it only from the inlined ledger digest for cycles 0001–0010 and did not open the previous insights file.

The main points:

- **Recurring wins:** stash-hit admission and landing rules (MRU landing, second-hit gate) gave hit-rate gains, and the probing tier does best near 128 slots. Shrinking it lost hit-rate, growing it by 128 helped, and growing to 256 lost again.
- **Recurring failures:** reuse-counter refinements were flat across several cycles. Recency gates, victim-preference rules, and shortcuts that remove the demotion or eviction path all lost hit-rate. Many refactors matched their parent exactly, and their claimed speed or memory benefits were never measured.
- **Open questions:** whether the probing optimum is exactly 128 or sits between 128 and 256, and whether any gain holds on a second workload or capacity (none has been tested). Per-op-time and metadata-size were never recorded. Several "Winner" labels (0005 v001, 0007 v002, 0008 v002, 0010 v003) sit on flat or lower hit-rate, so the baseline needs checking before those lineages are built on.
- **Per-policy notes:** every cycle tests one tiered design on the LIRS trace at size 2048, so the ledger cannot support a per-policy comparison.


---

- finished: 2026-10-20T15:04:47+00:00
- duration: 16.37s
- status: ok
- input_tokens: 4
- output_tokens: 3039
- cache_read_tokens: 41126
- cache_write_tokens: 14237
- cost_usd: 0.004779
