# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0001/cross-cycle
- allow_edits: True
- started: 2026-10-21T04:13:33+00:00

## timeline

### 04:13:38 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0001/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Changing *where* a promoted item lands (MRU insertion) beat changing *how many* items move. Only one cycle of evidence so far; treat as a lead, not a settled pattern.\n- Recency-skew rationale (don't make just-hit items the next victim) held up.\n\n## Recurring failures\n\n- Adding movement (more promotions, bidirectional probing<->target swaps) gained little or lost. Likely cause: demotion cascades toward stash/eviction offset the benefit.\n- Relaxing promotion eligibility alone gave only marginal gain; spare probing capacity is not a binding constraint.\n\n## Open questions\n\n- Does MRU placement generalize beyond workload-a, and does it cost per-op time?\n- Does combining MRU insertion with eligibility relaxation add anything, or was the marginal gain noise?\n- Do insertion-position changes on other paths (demotion, cascade to stash, stash->target vs. probing retries) help?\n- Metadata size and per-op time have never been measured; cost claims are impossible until they are in the fitness vector.\n\n## Per-policy notes\n\n- Dash segment promotion (`DashSegment::get`/`promotion_get`): the insertion-position lever is the productive direction; movement-count levers are not. Baseline comparisons so far cover hit-rate only.\n"
}
```

### 04:13:41 chat

I wrote `cross-cycle-insights.md` to the output dir, with the four required sections in order. The ledger has only one completed cycle, so the wins are marked as leads rather than settled patterns.

- **Recurring wins:**
  - Changing where a promoted item lands (MRU insertion) beat changing how many items move.
  - The recency-skew rationale held up.
- **Recurring failures:**
  - Extra movement (more promotions, bidirectional probing↔target swaps) gained little or lost, likely because of demotion cascades.
  - Relaxing eligibility alone gave only a marginal gain.
- **Open questions:**
  - Does MRU placement generalize beyond workload-a, and does it cost per-op time?
  - Does combining it with eligibility relaxation add anything?
  - Do other insertion paths matter?
  - Metadata size and per-op time have never been measured.
- **Per-policy notes:** insertion position is the productive lever for Dash segment promotion, and only hit-rate has been compared so far.


---

- finished: 2026-10-21T04:13:42+00:00
- duration: 8.90s
- status: ok
- input_tokens: 4
- output_tokens: 909
- cache_read_tokens: 33665
- cache_write_tokens: 6034
- cost_usd: 0.0368
