# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0002/cross-cycle
- allow_edits: True
- started: 2026-10-29T13:24:23+00:00

## timeline

### 13:24:39 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0002/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- The stash reuse gate holds up as the main lever for hit-rate. It was the strongest winner in 0001, and in 0002 its plain two-hit threshold turned out to carry almost all of the gain. Admitting or keeping items based on evidence of reuse is the only direction that has raised hit-rate.\n- Cutting multi-bucket moves (promotions plus demotion cascades) improves both hit-rate and per-op time. Changes that remove churn win, and changes that add churn lose even when each insert gets cheaper.\n- A lean common hit path pays off in median get latency. Removing per-hit counter writes and comparisons recovered the median get cost the gate had added in 0001.\n- Large cache sizes are where the hit-rate is won, with hm_0 the clearest case. Once churn stops there is room for a reused working set.\n\n## Recurring failures\n\n- Placement based only on free space (balanced placement, and later a version limited by a reuse flag) never moved hit-rate in either cycle. Its latency gain from 0001 is now in doubt.\n- Adding further \"protect reused items\" mechanisms on top of the gate (MRU demotion landing, reuse flags) is mostly redundant. They brought small or no gains because the gate already captures that reuse signal.\n- Any bookkeeping on the target-hit path (counter reads or writes, comparisons, conditional stores) steadily costs median get. That cost often eats the benefit of the change.\n- MRU landing for promotions adds promotion churn and slows ops. This holds because insert position and promotion speed are coupled through the hottest-in-target → probing rule.\n\n## Open questions\n\n- How noisy is the per-op-time measurement? Re-running an identical baseline is needed before small latency differences are trusted. That includes past latency \"wins\", which were judged on this measurement.\n- Can a write-free victim guard (a single bit, or one active only at large sizes) win back the large-cache hit-rate lost when the victim comparison was dropped?\n- What causes the put p99 rise under the threshold-only gate? Is it stash churn from unconditional promotion into a full target?\n- What limits web_0 at the large size, if placement does not?\n- Should the gate threshold grow with cache size?\n- Would requiring reuse before target → probing promotion cut churn without hurting hit-rate?\n\n## Per-policy notes\n\n- Dash segment (stash / target / probing tiers): every variant so far changed promotion or placement within the segment and left Item layout alone. The current baseline is the threshold-only stash gate with a lean hit path (cycle 0002, v001).\n- Gate parts priced separately: the threshold gives the hit-rate. The per-hit counter writes cause the median get cost. The victim comparison gives a small hit-rate gain at large sizes, mainly on hm_0, for a small get cost.\n- Free-slot balanced placement: retire it as a hit-rate strategy. Keep it only for latency, and only after the noise check, with free slots kept for hot items rather than cold evictees.\n- MRU landing: promotion landing hurts through churn. Demotion-only landing gives at most a small gain, at web_0 large, and is redundant with the gate.\n- Untried: Item layout changes, and gating promotion from target to probing.\n"
}
```

### 13:24:45 chat

I wrote `cross-cycle-insights.md` in the cross-cycle output dir with the four required sections in order. It is built only from the ledger digest for cycles 0001 and 0002, and I didn't read the previous file.

- **Recurring wins:** the stash reuse gate is the only thing that has raised hit-rate, and its plain two-hit threshold carries nearly all of that gain. Changes that cut multi-bucket moves were faster, and in 0001 they also won on hit-rate. Keeping the common hit path free of bookkeeping won back median get latency. Large cache sizes, especially hm_0, are where hit-rate is won or lost.
- **Recurring failures:** placing items by free space alone never moved hit-rate. Extra "protect reused items" mechanisms on top of the gate added little or nothing. Any per-hit counter work costs median get, and MRU landing on promotion slows things through extra churn.
- **Open questions:** the main one is whether the per-op-time benchmark is too noisy to trust small latency differences, including past latency wins. Others: whether a cheap, write-free victim guard can recover large-cache hits, what caused the put p99 rise, what limits web_0 at large size, and whether the gate threshold should grow with cache size.
- **Per-policy notes:** these cover the Dash segment. The current baseline is cycle 0002's v001: the threshold-only gate with a lean hit path. Each part of the gate is priced separately. Free-slot placement should be retired for hit-rate and kept for latency only if a noise check supports it. Item layout changes and requiring reuse before target-to-probing promotion haven't been tried yet.


---

- finished: 2026-10-29T13:24:46+00:00
- duration: 22.32s
- status: ok
- input_tokens: 4
- output_tokens: 1768
- cache_read_tokens: 36122
- cache_write_tokens: 7409
- cost_usd: 0.101872
