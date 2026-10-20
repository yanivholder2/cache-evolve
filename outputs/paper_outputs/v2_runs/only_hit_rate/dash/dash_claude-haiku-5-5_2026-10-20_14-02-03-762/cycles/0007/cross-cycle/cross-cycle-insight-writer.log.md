# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-10-20T14:48:21+00:00

## timeline

### 14:48:28 thinking



### 14:48:34 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Landing a re-referenced stash item at the MRU end of its target bucket is the only placement change that has held up. It was the winner in cycle 1 and has been the base for later stacked variants.\n- Requiring a second hit before a stash item is promoted is the one filter that showed a hit-rate gain on its own (cycle 2). It kept its small gain when stacked on MRU landing (cycle 3).\n- Admission and promotion filters that look at reuse help more than filters that change where items sit. Movement-changing edits have mostly hurt.\n- Gating on persistent reuse (the 2-bit saturating counter, cycle 6) is the only reuse-tracking idea that showed any hit-rate signal, but the signal was marginal.\n\n## Recurring failures\n\n- Moving items out of their tier and back is harmful. Removing and re-inserting probing-tier hits (cycle 1) pushed proven-hot keys back through target competition and lost ground.\n- Skipping the target retry for foreign evictions lowered hit-rate (cycle 1). Restoring it only when the stash is full recovered only part of the loss (cycle 2), so the shortcut is not worth pursuing as is.\n- Using the seen-in-stash marker as the eviction criterion hurt (cycle 3), because it discards recently arrived items before they can prove reuse.\n- Early readmission of recently evicted keys through a ghost-history fast path gave a slight hit-rate loss (cycle 5).\n- Many small refinements are flat against their parent and should not be kept as leads: occupancy-conditional landing (cycle 2), occupancy-conditional promotion gate (cycle 4), earliest-landed stash victim selection (cycle 4), demotion placement at the LRU end (cycle 5), gate reordering (cycle 6), back-to-back reuse filter (cycle 7), reset-by-halving on demotion (cycle 7), and the probing-tier zero-hit admission filter (cycle 7).\n- Changes that only alter layout or speed, such as the spare-bit marker (cycle 4), the occupancy counter (cycles 5 and 6), and the slot scan replacement (cycle 6), show no hit-rate change. Their benefit has never been measured.\n\n## Open questions\n\n- Does the second-hit promotion gate with MRU landing hold on workloads and capacities other than the LIRS trace at size 2048? Every cycle so far has used that single trace.\n- What are the per-op-time and metadata-size costs of the promotion gate, its marker, and the saturating reuse counter? No cycle has recorded either metric.\n- Winner labels do not match the reported evidence. Cycle 2's v001 matched its parent, cycle 4's v001 was unchanged from its parent, cycle 5's v001 was slightly below its parent, and cycle 7's v002 was within noise. Does the baseline for \"Winner\" need to be stated explicitly?\n- Several insights describe intended changes rather than verified overlay diffs. Is the marker really packed into a spare bit, and is the counter reset on demotion and eviction implemented as specified?\n- The rationale for the Front/Back default is described inconsistently across cycles. Which description is authoritative?\n\n## Per-policy notes\n\n- Stash tier (LIRS-style, size 2048): MRU landing and the second-hit gate are the only stash-side levers with any measured hit-rate signal. Victim-order and occupancy-conditional variants have shown no effect. Keep the MRU and second-hit combination as the base and test it on more workloads.\n- Probing tier: moving hits out and back is harmful, and the zero-hit admission filter was flat. Do not pursue further probing-tier movement without a measured cost offset.\n- Normal tier and eviction path: skipping the target retry for foreign evictions lowered hit-rate, and demotion placement changes were flat. The eviction path needs stash-capacity evidence before it is revisited.\n- Reuse metadata (seen-in-stash marker, 2-bit saturating counter): the marker works as an admission signal but not as an eviction signal. The saturating counter is the only reuse-tracking lead, and it needs a second workload and cost measurements before it is kept.\n"
}
```

### 14:48:37 chat

I wrote `cross-cycle-insights.md` to the cross-cycle output directory for cycle 0007. It has the four required sections in order, built only from the ledger digest. I did not read the previous version of the file.

The main points:

- **Recurring wins:** MRU landing for re-referenced stash items and the second-hit promotion gate are the only changes with a hit-rate signal that has held up across cycles. The saturating reuse counter showed a signal in cycle 6, but it was marginal.
- **Recurring failures:** Moving items out of the probing tier and back, and skipping the target retry for foreign evictions, both hurt hit-rate. Many small refinements were flat against their parent. Layout-only changes were never measured for cost.
- **Open questions:** Every cycle used the same LIRS trace at size 2048, so generalisation is untested. Per-op-time and metadata-size have never been recorded. Several "Winner" labels do not match the reported results, and several insights describe intended changes rather than verified diffs. The Front/Back default is documented inconsistently.
- **Per-policy notes:** Grouped by stash tier, probing tier, normal tier and eviction path, and reuse metadata.


---

- finished: 2026-10-20T14:48:37+00:00
- duration: 16.26s
- status: ok
- input_tokens: 4
- output_tokens: 2869
- cache_read_tokens: 38731
- cache_write_tokens: 11750
- cost_usd: 0.004172
