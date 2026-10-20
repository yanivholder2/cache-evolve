# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0002/cross-cycle
- allow_edits: True
- started: 2026-10-20T15:30:28+00:00

## timeline

### 15:30:38 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0002/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Keeping the small queue's one-hit-wonder filter intact is what pays off. The 10% small queue with the baseline promotion threshold has held up as the best point so far.\n- Giving main-eviction history its own bounded ghost (separate from the small-queue demotion ghost) is the only change that has shown a real, modest hit-rate gain. It cost some metadata and put latency.\n\n## Recurring failures\n\n- Changes that let more one-hit-wonders into main, or take capacity from main, lose hit rate. This covers a looser promotion threshold (freq>=1) and a larger small queue (15%).\n- Sharing one ghost between small-queue demotions and main evictions dilutes the filter and lowers hit rate.\n- Latency is not a usable selection signal. Large per-op latency shifts keep appearing on edits that add no hot-path work, so regressions have likely been promoted through noise.\n- Infrastructure failures (for example a Maven 429) can hide ideas. A rejection caused by infrastructure says nothing about the idea itself.\n\n## Open questions\n\n- Is the separate main-ghost's hit-rate gain larger than run-to-run noise, and does it hold on other traces and cache sizes?\n- What ghost size is best, and does re-entry from the ghost need a freq head start? Is the metadata and put-latency cost justified?\n- Is the small-ratio optimum below 10%? 5% and lower are untested.\n- Can repeated runs or multiple traces separate real effects from noise?\n- All evidence comes from one trace (workload-a) at one cache size, so none of it is shown to generalize.\n\n## Per-policy notes\n\n- S3-FIFO: Current best base is the 10% small queue, baseline promotion threshold, and a separate bounded main-eviction ghost. Next steps are to tune the ghost, sweep small ratios at 5% and below, and judge on hit rate over more traces.\n- S3-FIFO dead ends: ghost recycling that shares the demotion ghost, freq>=1 promotion, and small-queue ratios of 15% or more.\n"
}
```

### 15:30:45 chat

I wrote `cross-cycle-insights.md` to the cycle 0002 cross-cycle output directory. It has the four required sections in the fixed order and is built only from the cycle 0001 and 0002 ledger entries.

- **Recurring wins:** a 10% small queue with the baseline promotion threshold has held up as the best point so far. A separate bounded ghost for main evictions is the only change with a real, modest hit-rate gain, at some cost in metadata and put latency.
- **Recurring failures:**
  - Letting more one-hit-wonders into main hurts hit rate. This covers the freq>=1 promotion threshold and the 15% small queue.
  - Sharing one ghost between small-queue demotions and main evictions also lowers hit rate.
  - Latency is not a trustworthy selection signal, because large shifts appear on edits that add no work.
  - Infrastructure failures such as the Maven 429 can hide ideas without testing them.
- **Open questions:**
  - Is the separate ghost's gain larger than run-to-run noise?
  - What ghost size is best, and does re-entry need a freq head start?
  - Is the best small-queue ratio below 10%?
  - Does any of this hold beyond the one trace (workload-a) at one cache size?
- **Per-policy notes:** S3-FIFO is the only policy so far. The notes give the current best base (10% small queue, baseline threshold, separate main-eviction ghost), the next steps, and the dead ends.

I kept the numbers to those in the ledger and described everything else qualitatively.


---

- finished: 2026-10-20T15:30:45+00:00
- duration: 17.14s
- status: ok
- input_tokens: 4
- output_tokens: 1342
- cache_read_tokens: 35162
- cache_write_tokens: 6532
- cost_usd: 0.043362
