# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-10-20T14:36:31+00:00

## timeline

### 14:36:39 thinking



### 14:36:44 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Filtering which stash hits get promoted is the most consistent hit-rate lever. Requiring a second hit before promotion (cycle 2) and stacking it on MRU landing (cycle 3) both gave gains, though only on one trace.\n- MRU landing for re-referenced stash items is the one landing change that helped (cycle 1). Refining it by occupancy (cycle 2) added nothing.\n- Keeping proven-hot keys resident in the probing tier matters. Changes that remove and re-insert them (cycle 1) lost ground.\n\n## Recurring failures\n\n- Changes that add movement or remove a retry on the demotion path lower hit-rate. Remove-and-reinsert of probing hits (cycle 1) and unconditional direct-to-stash eviction without the target retry (cycle 1) both regressed. Gating the shortcut on free stash capacity (cycle 2) recovered only part of the loss.\n- Using the seen-in-stash marker as an eviction signal hurts. Preferring never-re-hit stash entries as victims discards recently arrived items that have not had time to prove reuse (cycle 3).\n- Admitting recently evicted keys early hurts slightly. The ghost-history fast path (cycle 5) let returning keys skip the seen-in-stash wait and lost hit-rate.\n- Small tweaks to landing slot, promotion gate, victim order, and demotion placement have shown no hit-rate effect. These include occupancy-conditional landing (cycle 2), occupancy-conditional gating (cycle 4), earliest-landed victim selection (cycle 4), and LRU-end demotion placement (cycle 5). Stop spending cycles on this class of tweak.\n- Layout or bookkeeping changes that leave hit-rate unchanged (spare-bit marker in cycle 3 and 4, occupancy counter in cycle 5) have not shown any realized saving. Their cost benefit is unmeasured.\n\n## Open questions\n\n- Winner status is inconsistent. Cycle 2's v001 matches its parent, cycle 4's v001 shows no change, and cycle 5's v001 shows a slight hit-rate drop. It is unclear what baseline each Winner is measured against.\n- Per-op-time and metadata-size have never been measured in any cycle. The cost side of every change, including the marker, the ghost set, and the occupancy counter, is unknown.\n- Does the MRU landing plus second-hit gate hold on workloads and capacities other than the LIRS trace at size 2048? All wins so far rest on that single trace.\n- Several insights rest on hypothesis text rather than the overlay diff (cycles 2, 4, 5). Implementations are not confirmed as described.\n- Does the foreign-eviction target retry pay for itself if stash capacity is handled better? Cycle 2 suggests stash pressure is what made it valuable.\n- Rationale for the Front/Back default is described inconsistently across cycle 1 insights, which makes v002's intent hard to verify.\n\n## Per-policy notes\n\nThe ledger does not name separate eviction policies. The notes below group observations by mechanism, and the only workload named is the LIRS trace.\n\n- Probing tier: keep hits resident. Removing and re-inserting probing hits hurts. Demotion placement from this tier has shown no effect.\n- Stash tier: promotion filtering and MRU landing are the only levers that moved hit-rate. Victim-selection changes (never-re-hit preference, earliest-landed) either hurt or had no effect. Capacity-gated shortcuts are a secondary lever whose cost is unmeasured.\n- Normal / target-bucket tier: the target retry on foreign eviction has some value under stash pressure. Removing it unconditionally hurts. No measured variant has changed target competition in a way that helped.\n- Ghost history: early readmission hurts on the one trace measured. Its memory cost is unknown.\n"
}
```

### 14:36:47 chat

I wrote `cross-cycle-insights.md` to the cycle-0005 cross-cycle output directory, using the four required sections in order. I did not read the previous version of the file.

The main signals across cycles 1–5:

- **Recurring wins:** Filtering which stash hits get promoted (a second-hit gate, plus MRU landing) is the most consistent hit-rate lever. Both results come from a single LIRS trace.
- **Recurring failures:** Changes that add movement or remove the demotion-path retry lower hit-rate. Using the seen-in-stash marker as an eviction signal hurts, and early readmission via ghost history loses slightly. Several small landing, gate, victim-order, and demotion tweaks showed no effect.
- **Open questions:** Per-op-time and metadata-size have never been measured. The Winner labels don't match their measured results (cycles 2, 4, and 5), so the baseline is unclear. Several insights rest on hypothesis text rather than the overlay diff.
- **Per-policy notes:** The ledger doesn't name separate eviction policies, so I grouped observations by tier (probing, stash, target bucket) plus the ghost history.


---

- finished: 2026-10-20T14:36:47+00:00
- duration: 16.39s
- status: ok
- input_tokens: 4
- output_tokens: 3052
- cache_read_tokens: 37229
- cache_write_tokens: 10435
- cost_usd: 0.003986
