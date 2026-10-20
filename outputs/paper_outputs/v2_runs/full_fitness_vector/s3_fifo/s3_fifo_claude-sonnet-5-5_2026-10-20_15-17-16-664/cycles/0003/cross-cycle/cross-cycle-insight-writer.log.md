# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0003/cross-cycle
- allow_edits: True
- started: 2026-10-20T15:34:09+00:00

## timeline

### 15:34:23 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Changes with a clear mechanism are the ones to credit. The scan consolidation (Cycle 3) removed redundant linear scans and left hit rate and metadata unchanged.\n- Preserving the small queue's one-hit-wonder filtering works. A separate bounded main-eviction ghost (Cycle 2) gave a modest hit-rate gain. It succeeded where recycling into the shared ghost (Cycle 1) failed.\n- Constant-only small-queue ratio changes are cheap. The gain has flattened: 10% helped modestly and 5% only marginally. The knob is near exhaustion.\n\n## Recurring failures\n\n- Latency is not a trustworthy selection signal. In every cycle, variants with no hot-path mechanism showed latency shifts, so regressions in hit rate were promoted by noise (ghost recycling, freq>=1 promotion, the 15% ratio, the rotation cap).\n- Letting more one-hit-wonders into main, or taking capacity from main, lowers hit rate. Examples: looser promotion, a larger small queue, and recycling main evictions into the shared ghost.\n- Guards on already-bounded behavior (the rotation cap, given a saturating frequency counter) cost hit rate and buy nothing.\n- Infrastructure failures (a Maven 429) can hide ideas. Do not treat the lost variant as evidence against its idea.\n- Every cycle has used one trace (workload-a) at one cache size, so no conclusion is shown to generalize.\n\n## Open questions\n\n- Is the separate main-ghost gain larger than run-to-run noise? How should the ghost be sized, and does it need a freq head start on re-entry? Is its metadata and put cost worth it?\n- Does the 5% small-queue ratio hold on other traces and cache sizes, or is the optimum workload-dependent?\n- Can an index such as a hash map replace the linear membership scans that dominate `put` cost?\n- Can the tests deleted by the scan refactor be restored without losing its gains?\n- Can repeated runs or multiple traces separate real effects from noise in the fitness signal?\n\n## Per-policy notes\n\n- S3-FIFO (small/main/ghost):\n  - The baseline promotion threshold is retained. The freq>=1 variant lost.\n  - The small ratio is around 5-10%. Larger ratios lose.\n  - The shared demotion ghost must stay clean. Main evictions belong in a separate ghost.\n  - The frequency counter saturates low, so rotation is already bounded.\n  - The next steps are scan and index work for `put` cost, and tuning the separate ghost. Judge by hit rate across traces.\n"
}
```

### 15:34:30 chat

I wrote `cross-cycle-insights.md` to the output dir, using the four fixed sections in the required order. I built it only from the three-cycle ledger digest and did not read the previous file.

- **Recurring wins:** Credit changes that have a clear mechanism, like the Cycle 3 scan consolidation. A separate main-eviction ghost gave a modest hit-rate gain. The small-queue ratio knob is flattening out.
- **Recurring failures:** Latency is an unreliable selection signal, because it moved even in variants that added no hot-path work. Changes that let more one-hit-wonders into main, or take capacity from it, lost hit rate. The rotation cap bought nothing. Every cycle used one trace at one cache size, and the one infrastructure failure (a Maven 429) hid an idea rather than refuting it.
- **Open questions:** Is the separate ghost's gain real, and how large should it be? Does 5% hold on other traces? Can a hash-map index cut `put` cost? Can the deleted tests be restored? Can repeated runs separate real effects from noise?
- **Per-policy notes:** These cover S3-FIFO only: keep the baseline promotion threshold, keep the small ratio around 5–10%, keep the shared demotion ghost clean, and treat rotation as already bounded.

I used no numbers beyond the ratios named in the digest.


---

- finished: 2026-10-20T15:34:30+00:00
- duration: 20.90s
- status: ok
- input_tokens: 4
- output_tokens: 1477
- cache_read_tokens: 35872
- cache_write_tokens: 7440
- cost_usd: 0.048486
