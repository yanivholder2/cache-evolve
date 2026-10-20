# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0003/cross-cycle
- allow_edits: True
- started: 2026-10-20T14:25:33+00:00

## timeline

### 14:25:39 thinking



### 14:25:43 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Landing position beats movement: placing a re-referenced stash item at the MRU end of its target bucket is the one change that has helped, and it still anchors the current best line.\n- Admission filtering on stash promotion helps: requiring a second hit before promotion reduces one-off stash hits displacing resident items. Its gain is small and so far appears on top of MRU landing rather than alone.\n\n## Recurring failures\n\n- Moving proven-hot items out of the probing tier hurts. Removing and re-inserting probing hits pushes them back through target competition and loses hit-rate.\n- Shortening the demotion path without compensation does not pay. Skipping the target retry for foreign evictions lowered hit-rate, and gating it on stash capacity recovered only a small part of the loss, which suggests the removed retry mattered under stash pressure.\n- Refinements to the landing slot that depend on target-bucket occupancy show no measurable effect, and there is no evidence the full-bucket case they target occurs on the tested workload.\n- Using the \"seen in stash\" marker as an eviction signal hurts. Preferring never-re-hit stash entries as victims discards recently arrived items that have not yet had a chance to prove reuse.\n- Marker relocation into a spare tag or bitmap bit produced no hit-rate change, and it is unclear whether the layout actually changed. Treat it as unverified, not as a neutral result.\n\n## Open questions\n\n- Do the MRU landing and the second-hit gate hold on workloads and capacities other than the single LIRS trace used so far?\n- Per-op-time and metadata-size have not been measured in any variant. Does the promotion filter or its marker carry a real cost?\n- Is v001 of the latest cycle a win against the baseline, or only against its parent? Its reported result matches the parent.\n- Would the removed target retry pay for itself if the shorter eviction path were paired with better stash capacity handling?\n- Is the victim-preference idea dead in general, or does it fail only when the marker is the sole eviction criterion?\n- Do several insights rest on hypothesis text rather than verified diffs, so the implementations as described are not confirmed?\n\n## Per-policy notes\n\n- Stash tier (promotion and landing): MRU landing for stash hits is the most consistent positive signal. Pair it with a second-hit promotion gate and test that pairing across more workloads and capacities.\n- Stash tier (eviction): marker-based victim preference hurts; avoid it unless combined with other criteria that protect recent arrivals.\n- Probing tier: keep resident probing hits in place; do not remove and re-insert them.\n- Normal/target tier (foreign evictions): skipping the target retry is a regression on its own. Revisit only together with stash capacity handling, and only with per-op-time measured.\n- Cost dimensions (all tiers): per-op-time and metadata-size are unmeasured across all cycles, so no policy has a known cost profile yet.\n"
}
```

### 14:25:46 chat

I wrote `cross-cycle-insights.md` to the output directory, built only from the inlined ledger digest for cycles 0001–0003. I did not read the previous version of the file.

It has the four required sections in order:

- **Recurring wins:** MRU landing for re-referenced stash items, and the second-hit promotion gate, which reduces one-off stash hits displacing resident items. The gate's gain is small and so far only measured on top of MRU landing.
- **Recurring failures:** removing and re-inserting probing hits; skipping the target retry for foreign evictions (gating it on stash capacity recovered only a small part of the loss); occupancy-conditional landing, which had no measurable effect; and using the "seen in stash" marker as an eviction signal, which discards recently arrived items. Marker relocation into a spare bit is listed as unverified, because the digest doesn't show whether the layout changed.
- **Open questions:** whether the wins generalize beyond the single LIRS trace, whether per-op-time and metadata-size have any cost, whether v001 beats the baseline or only its parent, and whether the removed retry pays off with better stash capacity handling.
- **Per-policy notes:** grouped by stash promotion, stash eviction, probing tier, normal/target tier, and cost dimensions.

No numbers were added beyond what the digest states.


---

- finished: 2026-10-20T14:25:46+00:00
- duration: 13.10s
- status: ok
- input_tokens: 4
- output_tokens: 2331
- cache_read_tokens: 35781
- cache_write_tokens: 8212
- cost_usd: 0.003166
