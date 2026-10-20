# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-10-20T14:27:56+00:00

## timeline

### 14:28:02 thinking



### 14:28:08 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Eviction-neutral representation changes (sentinel index, relink skip on MRU hit, lookup collapse, hasher swap, index pre-sizing) have consistently held hit-rate equal to their parent on workload-a at size 2048. The behaviour-preservation property is reliable; no size or per-op gain has been measured for any of them.\n\n## Recurring failures\n\n- Changes that add structure to the recency list have not paid off on hit-rate: the probation/protected split (v001 in 0005, v003 in 0004 with no clear direction) and the second-chance referenced-bit eviction (0005) both lowered or failed to show a gain.\n- Second-hit promotion (0006 v002) lowered hit-rate on the same trace.\n- Tail insertion of new keys has moved hit-rate in both directions: a gain in 0005 (v003), then losses in both stacked forms in 0006 (v001, v002). It is not a reliable win yet.\n- Ghost-based admission filters (0002 v002, 0003 v002) reported no hit-rate movement, and no cycle showed that the filter engaged on the trace. The result may be a non-test rather than a real null.\n- Two-of-three-variant failures in 0001 and one in 0004 were infrastructure rejections (session limit, missing insight file). These say nothing about their strategies and should not be counted as failures of the approach.\n- Named Winners with no measured gain or with a hit-rate dip (0003 v002, 0005 v002, 0006 v001) recur. Selection is not tracking the measured dimensions.\n\n## Open questions\n\n- Per-op time and metadata size have never been recorded on any lineage across six cycles. Until they are added to the fitness vector, no size or speed claim (including H01's and H03's) can be judged.\n- What is the selection criterion for Winners? Several were chosen without a hit-rate gain.\n- Does tail insertion help against the current parent, or did the 0005 gain depend on the parent it was measured against? The 0005 and 0006 results cannot be reconciled from the insights.\n- Does the H03 label refer to one mechanism? It has been mapped to a per-op check, a pre-sized index, a segmentation change, and a hasher swap. Fix one definition before lineages cite it.\n- Does any result generalise beyond workload-a at size 2048? All hit-rate evidence comes from one trace and one size, so scan-resistance claims are untested.\n- Lineage for the 0002 Winner's hit-rate gain is ambiguous; the parent it was measured against needs pinning down.\n\n## Per-policy notes\n\n- LRU (all cycles so far): every variant is LRU-based. Strict recency ordering appears to matter, since every change that altered eviction order either lowered hit-rate or showed no clear gain. Representation-only changes are safe but unproven as improvements.\n- LRU with tail insertion (0005, 0006): the only eviction-order change with any positive reading, but conflicting across cycles. Treat as unresolved.\n- LRU with segmentation / second-chance / second-hit promotion (0004, 0005, 0006): no evidence of gain on hit-rate; treat as dead as written unless a parameter change (protected cap, demotion rule) is tested on its own.\n- Admission (ghost filter, 0002–0003): engagement on the trace is unverified; no hit-rate effect recorded.\n- No non-LRU policy has been evaluated.\n"
}
```

### 14:28:10 chat

I wrote `cross-cycle-insights.md` in the cycle 0006 cross-cycle output dir, with the four fixed sections in order. I did not read any previous version of the file.

The main points:

- **Recurring wins:** Only one thing holds up across cycles: eviction-neutral representation changes keep hit-rate equal to their parent on workload-a at size 2048. None of them has a measured size or per-op gain.
- **Recurring failures:** Structural changes to the recency list (segmentation, second-chance, second-hit promotion) have not shown a gain. Tail insertion moved hit-rate up in cycle 0005 and down in both forms in 0006, so it is unresolved. Ghost-filter results may reflect the filter never engaging. Infrastructure rejections are noted as not telling us anything about their approaches.
- **Open questions:** Per-op time and metadata size have never been recorded. The selection criterion for Winners is unclear. The H03 label maps to different mechanisms across cycles. All hit-rate evidence comes from one trace at one size.
- **Per-policy notes:** Every variant so far is LRU, and no other policy has been evaluated.


---

- finished: 2026-10-20T14:28:11+00:00
- duration: 14.97s
- status: ok
- input_tokens: 4
- output_tokens: 2787
- cache_read_tokens: 37988
- cache_write_tokens: 10936
- cost_usd: 0.003961
