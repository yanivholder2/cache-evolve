# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-10-20T13:35:12+00:00

## timeline

### 13:35:18 thinking



### 13:35:24 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Reducing estimator error from colliding keys is the lever that has produced hit-rate gains: widening the sketch (Cycle 0004) and conservative update that raises only row-minimum counters (Cycle 0005) both tightened over-estimates.\n- The probation/protected split inside the main region improved hit-rate when it was introduced (Cycle 0003), and the current protected/probation ratio and back-of-probation demotion have held as the better settings against alternatives (Cycle 0004).\n- Layout-only sketch changes reliably preserve hit-rate, so they are safe to adopt, but they have not themselves produced gains.\n\n## Recurring failures\n\n- Changes that alter the frequency history the admission comparison relies on keep losing hit-rate: fixed-schedule halving (Cycle 0001) and global aging on a fixed access-count trigger (Cycle 0005). At this capacity the sketch needs its history intact.\n- Shrinking estimator depth undid the collision suppression that the min-across-rows estimate depends on (Cycle 0005).\n- Delaying admission behind a recency window in front of the LRU cost hit-rate and did not show the expected one-touch-burst benefit (Cycle 0002).\n- Key-to-counter mapping rewrites produced a small hit-rate dip (Cycle 0001) and have not been shown to be within noise.\n- Several Winners were advanced on claimed footprint or per-operation gains that the fitness vector does not measure, so they advanced by not regressing rather than by demonstrated improvement (Cycles 0001, 0002, 0003, 0004, 0005).\n- Narrowing the main region's protected segment and pushing demoted protected entries to the front of probation both lost hit-rate (Cycle 0004).\n\n## Open questions\n\n- Do the width and conservative-update gains hold on other workloads and cache sizes, and do they overlap or stack?\n- What do their metadata and per-operation costs look like once the fitness vector measures memory or timing?\n- Is there an aging trigger, not tied to access count, that sheds stale frequency without weakening admission?\n- Does the probation/protected gain hold beyond this trace, or is the ~4/5 protected ratio specific to it?\n- Does the sketch's saturation point ever bind on very hot keys, where the admission comparison could go insensitive?\n\n## Per-policy notes\n\n- Frequency sketch (TinyLFU estimator): width increases and conservative update reduce collision over-counting and are the productive direction; depth reduction, fixed-schedule halving, and global access-count aging hurt. Layout-only changes are neutral on hit-rate.\n- Admission: the comparison has been sensitive to any change that shifts when or how frequency history is seen; keep the sketch's history and the admission rule intact in the next cycle.\n- Window / entry path: a recency window ahead of the LRU, with admission applied at window exit, lost hit-rate and should not be re-proposed at this capacity.\n- Main region (SLRU-style): the probation/protected split is the one structural change that gained; the incumbent protected cap and back-of-probation demotion are the better settings; halved protected cap and push-to-front demotion are ruled out.\n- Caveat for all policies: every result so far comes from a single trace at a single cache size, so each gain is provisional until tested elsewhere.\n"
}
```

### 13:35:26 chat

I wrote `cross-cycle-insights.md` to the cycle 0005 cross-cycle output directory, with the four fixed sections in order. I built it only from the inlined ledger digest and did not read any previous version of the file.

The main signals across cycles 0001–0005:

- **Wins:** Reducing collision over-counting in the frequency sketch is the lever that has produced hit-rate gains (widening in 0004, conservative update in 0005). The probation/protected split in the main region gained in 0003, and the current ratio and demotion placement have held up as the better settings.
- **Failures:** Changes that alter the frequency history admission relies on keep losing hit-rate, including fixed-schedule halving and access-count aging. Reducing sketch depth, the recency window in front of the LRU, and the main-region variants from 0004 also lost.
- **Unmeasured payoffs:** Several Winners advanced on claimed memory or per-operation gains that the fitness vector does not measure, so they advanced by not regressing.
- **Caveat:** Every result comes from one trace at one cache size, so all gains are provisional.

No numbers were invented; dimensions are described qualitatively.


---

- finished: 2026-10-20T13:35:27+00:00
- duration: 15.37s
- status: ok
- input_tokens: 4
- output_tokens: 2508
- cache_read_tokens: 36954
- cache_write_tokens: 9623
- cost_usd: 0.003549
