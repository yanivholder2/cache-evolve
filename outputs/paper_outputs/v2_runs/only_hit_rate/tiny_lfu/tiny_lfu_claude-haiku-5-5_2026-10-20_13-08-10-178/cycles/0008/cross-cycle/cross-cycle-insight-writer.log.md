# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-10-20T13:47:53+00:00

## timeline

### 13:47:59 thinking



### 13:48:06 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Reducing the sketch's collision error is the most reliable lever. Widening the sketch (0004), adding a depth row (0007), and conservative update (0005) all moved hit-rate up, and the gains come from tightening over-estimates rather than from new memory.\n- Estimator refinements stack with diminishing returns. Conservative update added little on top of the width change (0006), so they likely act on the same error rather than independent sources.\n- Victim choice among probation's LRU-tail entries by sketch estimate helped (0008). Changing which probation entry leaves is the clearest recent win.\n- The protected/probation split is a real lever, but the useful range sits near the incumbent ratio. A larger protected share helped once (0006) and reversed when pushed further (0007); a halved cap also lost (0004).\n\n## Recurring failures\n\n- Fixed-schedule aging or halving of sketch counters discards history admission needs at this capacity (0001, 0005). Do not retry it on an access-count trigger.\n- Removing estimator depth hurt (0005), because min-across-rows needs enough independent rows to suppress collisions.\n- Keeping first-touch or one-touch keys out of the sketch's update path, or gating admission on the sketch, loses frequency signal (0006 doorkeeper, 0007 gated promotion). Changing when and where admission sees a new key cost hit-rate (0002 window-in-front-of-LRU).\n- Tie-rule changes against probation victims hurt (0008), because queue position was protecting older cold entries the sketch could not separate.\n- Ghost-history frequency bonus barely moved hit-rate (0008) and carries unmeasured metadata and per-op costs.\n- Demotion to the front of probation rather than the back lost hit-rate (0004).\n- Layout-only and allocation changes (0002, 0003) preserve hit-rate by construction. Their claimed footprint and per-op gains are on dimensions the fitness vector does not measure, so they advance only by not regressing.\n\n## Open questions\n\n- Do the width, depth, and conservative-update gains hold on other workloads and cache sizes? Every result so far is one trace at one size.\n- What do the sketch's extra rows, width, and conservative-update pass cost in counter memory and per-op time? The fitness vector does not report either, so none of these gains can yet be weighed against cost.\n- Where is the protected-share optimum on other traces, and does it sit near the current ratio?\n- Is the victim-by-estimate gain independent of the estimator changes it depends on, or does it overlap them?\n- Is there an aging trigger that is not an access count and that keeps admission intact?\n- Attribution is still shaky: some Winner records could not be inspected (0006, 0008), and 0007's Winner designation conflicted with its own record. Confirm which change actually advanced before building on it.\n- Does a wider tail scan beat a two-entry victim comparison?\n\n## Per-policy notes\n\n- **TinyLFU admission (frequency sketch):** Error-reduction changes (width, depth, conservative update) are the productive direction. Decay, halving, aging, and update-path filtering have all lost. Layout-only changes should be paired with a memory or timing signal before they count as wins.\n- **Main region (probation/protected LRU):** The split and its sizing matter. Hold the protected share at or near the incumbent ratio. Demotion should stay at the back of probation. Promotion should happen on every hit, not gated on sketch estimate.\n- **Victim selection:** Choosing the colder of the LRU-tail probation candidates by sketch estimate is the best recent change. Keep tie handling as-is. A wider tail scan is untested.\n- **Admission window / recency front:** A window ahead of the main LRU has lost hit-rate at this capacity (0002). Do not re-propose it in that form.\n- **Ghost history:** The frequency bonus for recently evicted keys showed no meaningful gain. Revisit only with a more selective signal and measured metadata cost.\n"
}
```

### 13:48:09 chat

I wrote `cross-cycle-insights.md` in the output directory, using the four fixed sections in order. I built it only from the inlined ledger digest and did not read any previous version of the file.

The main signals across cycles 0001–0008:

- **Recurring wins:** Reducing sketch collision error (width, depth, conservative update) has produced most of the hit-rate gains. Choosing probation victims by sketch estimate (cycle 0008) is the clearest recent win. The protected/probation split helps, but only near the incumbent ratio.
- **Recurring failures:** Fixed-schedule sketch aging, removing sketch depth, filtering first-touch keys out of the sketch, gating promotion on the sketch, and a window ahead of the main LRU all lost hit-rate. Layout-only changes preserve hit-rate, but their claimed footprint and timing gains are unmeasured.
- **Open questions:** Every result comes from one trace at one cache size. The memory and timing costs of the estimator changes are unmeasured. Winner attribution is unreliable for some cycles; cycle 0007's Winner designation conflicts with its own record.

Nothing in the file is a number I computed; all references are to cycles and qualitative directions.


---

- finished: 2026-10-20T13:48:09+00:00
- duration: 16.02s
- status: ok
- input_tokens: 4
- output_tokens: 2720
- cache_read_tokens: 39190
- cache_write_tokens: 12052
- cost_usd: 0.004163
