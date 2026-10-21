# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-10-21T04:27:39+00:00

## timeline

### 04:27:48 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Two wins in five cycles, both on workload-a: MRU placement on hit-promotion (cycle 1) and the second-sighting admission filter (cycle 3). Each came from a lever that changed what is kept or where hit entries land, without adding item movement.\n- Screening out one-hit-wonders at admission is the strongest lever found so far. Placement tweaks never matched it.\n\n## Recurring failures\n\n- Placement-position tweaks (insert, demotion, eligibility, two-tier insert) were flat after cycle 1. Every variant matched its parent's hit-rate. Either the levers are saturated or the overlays were dead edits.\n- Extra item movement (bidirectional swaps, more promotions) and delayed or relocated eviction (wider probe) gained nothing or lost a little, because of demotion cascades and cold entries crowding out hot ones.\n- Refinements of the admission filter (counters, victim-conditional exemption, doorkeeper, 4x capacity, stacking on another admission parent) all landed within noise or below the parent. Smaller or faster-resetting state lost accuracy. Overlapping admission mechanisms do not stack.\n- \"Winner\" labels from cycle 2 onward were mostly noise-level or relative to a weak cohort, so they are unreliable as evidence of improvement.\n\n## Open questions\n\n- Do the cycle 1 and cycle 3 gains hold on other workloads and cache sizes? Everything so far is workload-a.\n- What do the wins cost in metadata-size and per-op-time? Only hit-rate has ever been measured, so noise-level winners cannot be rejected on cost.\n- Are the overlay edits on the executed path? Repeated exact ties with the parent suggest some are not. Victim selection in particular is untested on a confirmed live path.\n- Is there an untried structural lever, such as bucket and probing structure or a different admission signal, that moves hit-rate more than noise?\n- Is the Winner rule relative to the cohort, or is the fitness vector missing dimensions?\n\n## Per-policy notes\n\n- Promotion (hit path): MRU insertion on hit helps. Eligibility relaxation and back-promotion add little or nothing.\n- Admission: the one-bit unconditional second-sighting filter (cycle 3 v002) is the incumbent, and tuning it is exhausted on workload-a.\n- Eviction and victim selection: a furthest-from-MRU scan tied its parent exactly, which is inconclusive. Cold-end demotion also showed no effect.\n- Probing and stash: a wider neighbourhood before the stash slightly hurt. Bucket and probing structure is otherwise unexplored.\n"
}
```

### 04:27:52 chat

I wrote `cross-cycle-insights.md` in the output dir, with the four required sections in order. I built it only from the five-cycle ledger digest. I didn't open any other files.

The main points it records:
- **Wins:** MRU placement on hit-promotion (cycle 1) and the second-sighting admission filter (cycle 3). Both are on workload-a.
- **Failures:** After cycle 1, placement tweaks, extra item movement, a wider probe, and every refinement of the admission filter landed flat or lower. Winner labels from cycle 2 on were mostly noise-level.
- **Open questions:** Whether the gains hold on other workloads and cache sizes. What they cost in metadata and per-op-time, since only hit-rate has been measured. Whether the overlay edits are on the executed path. Whether a structural lever such as bucket and probing structure is still untried.
- **Per-policy notes:** Promotion, admission, eviction and victim selection, and probing and stash each get a short entry. The one-bit admission filter is marked as the incumbent.


---

- finished: 2026-10-21T04:27:52+00:00
- duration: 12.84s
- status: ok
- input_tokens: 4
- output_tokens: 1370
- cache_read_tokens: 37232
- cache_write_tokens: 8796
- cost_usd: 0.05311
