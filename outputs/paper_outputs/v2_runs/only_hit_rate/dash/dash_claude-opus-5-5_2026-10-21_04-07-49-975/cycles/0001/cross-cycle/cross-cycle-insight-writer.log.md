# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0001/cross-cycle
- allow_edits: True
- started: 2026-10-21T04:15:27+00:00

## timeline

### 04:15:30 thinking



### 04:15:36 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0001/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- (Single cycle so far, so nothing is confirmed as recurring yet.) Making admission into the scarce normal tier stricter is the strongest signal. The stash→target edge needed two stash hits before promotion, and that variant beat every variant that changed placement.\n- Filtering at the stash edge appears cheap because the stash is larger than the normal tier and can hold unproven keys while they wait.\n\n## Recurring failures\n\n- (Single cycle; provisional.) Changes that loosen admission hurt hit-rate. This includes doing it indirectly: MRU landing moves an item to a hotter position, and that lowers the bar for promotion to probing.\n- Placement that relies on a free slot (spilling into an empty probing slot) rarely triggers, because normal buckets are almost always full at the same time in steady state.\n- Anything that adds normal-tier moves sets off cross-bucket demotion cascades and hurts neighbouring buckets too.\n\n## Open questions\n\n- Only hit-rate has been measured. Metadata-size and per-op-time are unknown, including the claimed get-latency benefit of the stash-hit filter.\n- Is two the best stash-hit threshold, or would an adaptive threshold do better?\n- Does a hit-count or recency filter on the target→probing edge give the same benefit as the stash filter?\n- Can MRU landing help if it is paired with a separate hit counter, so the bar for entering probing doesn't drop?\n\n## Per-policy notes\n\n- Tiered bucket policy (stash → target → probing/normal), the only policy explored so far:\n  - Which items get into the tier matters more than where they are placed inside a bucket or which bucket they go to.\n  - The \"is hottest\" promotion check reads bucket position. Any change to where an item lands also changes its promotion threshold, so these must be evaluated together.\n  - The current baseline to build on is the two-stash-hit admission filter.\n"
}
```

### 04:15:41 chat

I wrote `cross-cycle-insights.md` in the cross-cycle output dir, using the four required sections in order. The ledger has only one cycle, so I labelled the "recurring" wins and failures as provisional single-cycle signals.

- **Recurring wins:** making it harder to get into the normal tier pays off. The filter requiring two stash hits before promotion won, and it costs little because the stash is larger than the normal tier.
- **Recurring failures:**
  - Loosening admission lowered hit-rate. MRU landing does this indirectly, because a hotter position lowers the bar for moving to probing.
  - Spilling into a free slot rarely triggers, since the normal buckets are almost always full at once.
  - Any change that adds normal-tier moves also hurts neighbouring buckets through the demotion cascade.
- **Open questions:**
  - Only hit-rate has been measured; metadata-size and per-op-time are still unknown.
  - Is two the right stash-hit threshold?
  - Would a filter on the target→probing edge help in the same way?
  - Could MRU landing work if paired with a separate hit counter?
- **Per-policy notes:** these cover the tiered bucket policy (stash → target → probing/normal), the only one explored so far:
  - Which items get into the normal tier matters more than where they are placed.
  - Placement and promotion threshold have to be evaluated together.
  - The two-stash-hit filter is the current baseline.


---

- finished: 2026-10-21T04:15:42+00:00
- duration: 14.59s
- status: ok
- input_tokens: 4
- output_tokens: 1309
- cache_read_tokens: 33070
- cache_write_tokens: 6484
- cost_usd: 0.084904
