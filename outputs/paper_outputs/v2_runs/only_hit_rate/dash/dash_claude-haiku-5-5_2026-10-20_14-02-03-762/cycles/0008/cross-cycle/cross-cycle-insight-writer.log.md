# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-10-20T14:53:44+00:00

## timeline

### 14:53:51 thinking



### 14:53:57 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Landing stash-hit promotions at the MRU end of the target bucket is the one change that has moved hit-rate in the right direction (cycle 1), and it has been kept as the base for later stash work.\n- Requiring a second hit before a stash item is promoted gave a small hit-rate gain on top of MRU landing (cycles 2 and 3). The evidence is one trace at one capacity.\n- Behaviour-preserving refactors (occupancy counters, store-skip, gate reordering) have matched their parent exactly. That is a sound way to keep correctness, but it is not a performance win until cost is measured.\n\n## Recurring failures\n\n- Reuse-counter refinements have been flat for four cycles (cycles 6 to 8): saturating counters, back-to-back gating, halving on demotion, and saturated-gate promotion. None moved hit-rate beyond noise on LIRS workload-a at size 2048.\n- Changes that move or re-insert proven-hot items hurt. Removing and re-inserting probing-tier hits (cycle 1) pushed hot keys back through target competition.\n- Skipping the target retry for foreign evictions hurt hit-rate (cycle 1). Gating it on stash capacity recovered only part of the loss (cycle 2), so the retry matters under stash pressure.\n- Using the seen-in-stash marker as an eviction preference hurt (cycle 3). It discarded recently arrived items that had not yet had a chance to prove reuse.\n- Shrinking the probing tier to grow the stash lost hit-rate (cycle 8). Stash pressure does not look like the binding constraint on this trace.\n- Early readmission via a ghost-history fast path gave a slight hit-rate loss (cycle 5).\n- Landing-slot, victim-order, and occupancy-conditional refinements (cycles 2 and 4) showed no effect. Demoted-item placement (cycle 5) also showed no effect.\n\n## Open questions\n\n- Metadata-size and per-op-time have not been measured in any cycle. Every claimed cost benefit (occupancy counters, spare-bit marker, reuse-counter bits) is unverified. This is the largest gap.\n- Do the MRU landing and second-hit gate hold on workloads and capacities other than LIRS workload-a at size 2048?\n- Several \"Winner\" labels do not match their measured results (cycles 4, 5, 7, 8, and the inherited v001 in cycle 2). Is the label carried forward, or is it winning against a different baseline? Until this is clear, treat Winner status as weak evidence.\n- Many insights describe intended changes, not verified implementations, because overlay diffs were not readable. Implementations remain unconfirmed.\n- Would a larger probing tier, the opposite of the cycle 8 shift, help, or is the current split already reasonable?\n\n## Per-policy notes\n\n- Stash tier (promotion and landing): MRU landing plus a second-hit gate is the only lead with a hit-rate signal. Keep it, and test it more broadly.\n- Stash tier (victim selection): marker-based victim preference hurt; earliest-landed selection matched baseline. Victim-order tweaks have not helped.\n- Probing tier: remove-and-reinsert of hits hurt; zero-hit exit filtering and shrinking the tier did not help. Residency in probing appears valuable.\n- Normal tier and demotion: LRU-end placement and reuse-counter reset or halving on demotion were flat. No signal here yet.\n- Foreign eviction: skipping the target retry hurt; the retry matters under stash pressure. Occupancy-counter victim search matched parent, so it is a cost-only experiment.\n- Ghost history: readmission fast path lost slightly. Revisit only with a second workload showing a clear gain.\n"
}
```

### 14:53:59 chat

I wrote `cross-cycle-insights.md` to the cycle 0008 cross-cycle output directory, using the four required sections in order. I did not read the previous version of the file, as the instructions require.

The main points across cycles 0001–0008:

- **Recurring wins:** Landing stash-hit promotions at the MRU end of the target bucket (cycle 1) is the only change that moved hit-rate in the right direction. A second-hit gate on stash promotion added a small gain on top of it (cycles 2–3). Both rest on one trace at one capacity.
- **Recurring failures:** Four cycles of reuse-counter refinements (cycles 6–8) were flat on LIRS workload-a at size 2048. Moving or re-inserting hot items, skipping the foreign-eviction retry, using the marker as an eviction preference, and shrinking the probing tier all lost hit-rate.
- **Open questions:** Metadata-size and per-op-time were never measured, so every claimed cost benefit is unverified. Several "Winner" labels do not match their measured results (cycles 4, 5, 7, 8, and the inherited v001 in cycle 2). Many insights describe intended changes because the overlay diffs were not readable.
- **Per-policy notes:** Grouped by stash promotion, stash victim selection, probing tier, normal tier and demotion, foreign eviction, and ghost history.

The file is a fresh projection of the digest, so deleting and regenerating it from the same ledger should give an equivalent result.


---

- finished: 2026-10-20T14:54:00+00:00
- duration: 16.01s
- status: ok
- input_tokens: 4
- output_tokens: 3162
- cache_read_tokens: 39522
- cache_write_tokens: 12726
- cost_usd: 0.004522
