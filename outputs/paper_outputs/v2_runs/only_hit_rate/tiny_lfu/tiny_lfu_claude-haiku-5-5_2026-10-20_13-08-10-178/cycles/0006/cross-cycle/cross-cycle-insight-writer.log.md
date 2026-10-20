# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-10-20T13:39:18+00:00

## timeline

### 13:39:24 thinking



### 13:39:29 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Reducing estimator error is the most reliable lever. Widening the sketch (C4) and conservative update, which raises only row-minimum counters (C5), both gained hit-rate by tightening over-counting from collisions. Refinements on top of each other have diminishing returns, so they likely act on the same error.\n- Main-region segment sizing matters. The probation/protected split (C3) and a larger protected share (C6) were both hit-rate winners in the main region.\n\n## Recurring failures\n\n- Fixed-schedule aging or halving of sketch counters loses hit-rate (C1, C5). Discarding frequency history weakens the admission comparison at this capacity.\n- Changing when or whether admission sees a key loses hit-rate. A recency window ahead of the LRU (C2) and gating first-touch keys out of the sketch with a doorkeeper (C6) both lost. The withheld increments are not recovered by estimate-time correction.\n- Reducing sketch depth (C5) loses hit-rate. The min-across-rows estimate needs enough independent rows to suppress collisions.\n- Layout-only sketch changes (C1, C2, C3) hold hit-rate by construction, not by measured gain. Their claimed footprint and per-op benefits sit on dimensions the fitness vector does not measure, so they advance by not regressing.\n- Main-region variants that shrink the protected cap or push demoted entries to the front of the list lost hit-rate (C4). The incumbent ratio and back-of-probation placement are the better settings on this trace.\n\n## Open questions\n\n- Do the width, conservative-update, and protected-share gains hold on other workloads and cache sizes? Every result so far is one trace at one size.\n- What do the estimator and layout gains cost in memory and per-operation time? A timing or metadata dimension would let the Winners be confirmed rather than assumed.\n- Do the width and conservative-update gains overlap, or do they stack?\n- Is there an aging trigger, not tied to access count, that sheds stale frequency without weakening admission?\n- Is there a doorkeeper-style correction that matches the withheld increments closely enough to be worth retrying?\n\n## Per-policy notes\n\n- TinyLFU admission and frequency sketch: wins come from estimator error reduction (width, conservative update). Losses come from aging, depth reduction, and admission-timing or gating changes. Keep the sketch's history intact and treat admission as fixed.\n- Main-region segmentation (SLRU-style probation/protected within the main region): the protected-share and split changes are the gainful levers. Protected-cap halving and push-to-front demotion are not.\n- Window or LRU front-end (window-TinyLFU style): a recency window ahead of the LRU lost hit-rate at this capacity. Do not re-propose it in that form.\n- Layout and allocation (applies across policies): hit-rate neutral by construction. Keep layout changes out of stacking until a metadata or timing signal can confirm them.\n"
}
```

### 13:39:31 chat

I wrote `cross-cycle-insights.md` to the cycle 0006 cross-cycle output directory. It is regenerated from the six-cycle ledger digest only, and I did not read the previous version.

The four sections, in order:

- **Recurring wins:** Estimator error reduction (sketch widening in cycle 0004, conservative update in cycle 0005) and main-region protected-segment sizing (cycle 0003 split, cycle 0006 larger protected share).
- **Recurring failures:** Fixed-schedule sketch aging, admission-timing changes (the window in front of the LRU, the doorkeeper gate), sketch depth reduction, and layout-only changes whose claimed benefits are unmeasured.
- **Open questions:** Whether the gains hold beyond the single trace and cache size, their memory and per-op cost, whether width and conservative update stack, and whether a non-access-count aging trigger is viable.
- **Per-policy notes:** Grouped by TinyLFU admission and sketch, main-region segmentation, window/LRU front-end, and layout work.

The file contains no invented numbers.


---

- finished: 2026-10-20T13:39:32+00:00
- duration: 14.04s
- status: ok
- input_tokens: 4
- output_tokens: 2298
- cache_read_tokens: 37698
- cache_write_tokens: 10174
- cost_usd: 0.003561
