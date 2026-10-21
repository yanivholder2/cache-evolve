# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-10-20T20:17:35+00:00

## timeline

### 20:17:45 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Admission-side sizing is the only lever with consistent signal: a small probation queue (a few percent of capacity, well below the baseline) paired with a shortened ghost beats baseline across several cycles (0002, 0003, 0006, 0007, 0009).\n- Sizing alone explains the gains. Later bundles (main-evictee ghosting, hit-path promotion) added nothing once sizing was held fixed.\n- Ghost length acts as an admission filter: shorter (stricter) helps, longer hurts.\n- Simplest configuration wins ties: the sizing-only config matches the more complex variants.\n\n## Recurring failures\n\n- Ageing/decay changes to main (faster freq decay, lower freq cap, in-place ageing) lose every time. This workload rewards keeping reuse history. A higher cap gave at most noise-level gain.\n- Rewrites of main-eviction mechanics (lookahead, reinsertion caps) lose. The unbounded second-chance scan does protective work.\n- Loosening admission (looser promotion gate, pre-protected or boosted ghost re-admits, grace passes) loses by polluting main.\n- Filtering ghost hits or ghost content (freq>0 fast path, reuse-only ghosting) loses. Ghost hits are strong reuse evidence.\n- Promotion strictness/timing (eviction-time gate, hit-path promotion, thresholds, freq reset) ties parent repeatedly. These are either non-binding or no-op diffs.\n- Adaptive/feedback sizing lost to the fixed ratio.\n- Per-key ghost metadata and main insertion-position tweaks gave no signal.\n\n## Open questions\n\n- Where is the small-queue optimum? 3% vs 5% vs larger were never compared head to head on a shared parent.\n- Are exact ties real or no-op diffs? Variant authors cannot read overlay source, so behaviour change is unverified.\n- Do the sizing gains hold on a second trace and cache size, including scan-heavy workloads? All evidence is one trace at one size, and gains are small and may be noise.\n- Does ghosting main evictees add anything on top of sizing?\n- Per-op-time and metadata-size have never been measured, so cost is unknown, especially for hit-path work and extra ghost inserts.\n\n## Per-policy notes\n\n- S3-FIFO (the only policy so far): baseline structure looks near a local optimum on this trace. Remaining untested ideas: a sweep of the small ratio, validation on other traces, and structurally new levers that are verified to change behaviour.\n- Drop promotion thresholds, ageing/decay, adaptive sizing, ghost-content filters, and main-eviction rewrites from future hypotheses.\n"
}
```

### 20:17:48 chat

I wrote `cross-cycle-insights.md` to the output dir. It has the four fixed sections in order, built only from the nine-cycle ledger digest.

- **Recurring wins:** A small probation queue plus a shortened ghost is the only lever with consistent signal. The sizing-only config matches the more complex variants.
- **Recurring failures:** Ageing and decay changes, main-eviction rewrites, looser admission and ghost-hit filtering all lost. Promotion strictness and timing tied their parents repeatedly.
- **Open questions:**
  - Where the small-queue optimum sits, since 3% and 5% were never compared head to head.
  - Whether the exact ties are no-op diffs, since variant authors can't read the overlay source.
  - Whether the gains hold on a second trace and cache size.
  - Per-op-time and metadata-size are still unmeasured.
- **Per-policy notes:** Only S3-FIFO has been tried. It looks near a local optimum on this trace, and the file lists which ideas to drop from future hypotheses.


---

- finished: 2026-10-20T20:17:48+00:00
- duration: 13.22s
- status: ok
- input_tokens: 4
- output_tokens: 1397
- cache_read_tokens: 39895
- cache_write_tokens: 11487
- cost_usd: 0.064678
