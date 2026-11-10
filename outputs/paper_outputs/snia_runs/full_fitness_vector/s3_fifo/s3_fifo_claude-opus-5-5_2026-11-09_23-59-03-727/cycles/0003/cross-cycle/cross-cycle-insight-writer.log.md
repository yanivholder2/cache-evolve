# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0003/cross-cycle
- allow_edits: True
- started: 2026-11-10T04:09:46+00:00

## timeline

### 04:09:59 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- One change per variant. Cycles 1–2 bundled changes or moved a single trade-off knob, and they kept trading hm_0 gains for web_0 losses. Cycle 3 isolated each change and got the first cohort with no regression on either trace.\n- Taking away \"credit without reuse\" helps hm_0 and doesn't hurt web_0. Examples: ghosting only main victims that were hit while resident, and resetting freq on promotion. Filters that free ghost budget also lengthen the useful history at no cost.\n- Structural implementation changes earn timing credit and leave hit rates untouched. Two have worked so far: slab plus hash indexes (cycle 2), then a faster Fx-style hasher (cycle 3). They are the only per-op-time gains that their diffs explain.\n- Per-entry metadata has stayed flat in every cycle. Policy and structural changes haven't added per-entry cost so far.\n\n## Recurring failures\n\n- Fixed-ratio queue sweeps and resizing ghost regions only move the hm_0 vs. web_0-large trade-off around. The summary score never improves (cycles 1–2).\n- The ARC-style adaptive small/main split regressed. Its signals were unbalanced and the ghost bound was tied to a moving main size (cycle 1).\n- Loosening graduation to `freq >= 1` hurts web_0. Cycle 3 confirmed the threshold as the cause, and the freq reset was cleared. Keep `freq > 1`.\n- Per-op-time from policy-only diffs is noise. Unexplained timing shifts, in both directions, have shown up in all three cycles. Don't give timing credit without a structural cause.\n- Both early hypotheses about web_0's mechanism were wrong: ghost-slot competition and the threshold-only effect. This happened because changes were bundled.\n\n## Open questions\n\n- Do the cycle-3 Winners stack additively once merged? They are Fx hashing, reset-on-promotion, and reuse-gated main-victim ghosting.\n- What causes the rest of web_0's large-size gap? Candidates: the ghost bound relative to main size (the hidden coupled variable), or the small-queue share. Test one variable at a time.\n- Can the benchmark environment be pinned, with a same-machine baseline re-run, so that timing for policy diffs becomes trustworthy?\n- Should heap bytes per resident key supplement the per-entry metadata metric? Hashed structures and ghost key storage add memory that the current metric doesn't see.\n- Further structural speedups, e.g. using the entry API to cut hash probes per miss.\n\n## Per-policy notes\n\n- S3-FIFO (the only policy explored so far):\n  - hm_0 consistently rewards more main residency and history for reused keys.\n  - web_0, especially at the large size, is sensitive to looser graduation and to squeezing the probation window or the demotee history.\n  - The ghost bound is shared, zero-sum, and coupled to main size. Changes that alter main size or ghost contents without accounting for this have backfired.\n  - The current best base is slab plus hash, with Fx hashing, the `freq > 1` threshold, reset-on-promotion, and reuse-gated main-victim ghosts. That combination is still to be confirmed as additive.\n"
}
```

### 04:10:13 thinking



### 04:10:13 chat

I wrote `cross-cycle-insights.md` in the cycle-0003 cross-cycle output directory, built fresh from the three-cycle ledger with the four required sections in order.

The main points it carries forward:

- **Recurring wins:** Making one change per variant is what finally removed the hm_0 vs. web_0 trade-off. Cycle 3 was the first cohort with no regression on either trace. Two other things held up:
  - Removing credit for keys that were never reused in main helps hm_0 without hurting web_0.
  - Only structural changes (slab plus hash indexes, then the faster hasher) have shown timing gains their diffs explain.
- **Recurring failures:**
  - Fixed-ratio sweeps and resizing ghost regions only shift the trade-off between the two traces.
  - The ARC-style adaptive split regressed.
  - Loosening graduation to `freq >= 1` hurts web_0, so `freq > 1` stays.
  - Timing changes from policy-only diffs have been noise in every cycle.
- **Open questions:**
  - Whether the three cycle-3 Winners add up when merged.
  - What's left of web_0's large-size gap: the ghost bound or the small-queue share.
  - Pinning the benchmark environment so policy timing can be trusted.
  - Adding a heap-bytes-per-key metric, since the current one misses memory from hashing and ghost keys.
- **Per-policy notes:** S3-FIFO is the only policy tried so far. The notes list what each trace tends to reward and suggest a best base for the next cycle, which is still unconfirmed until the merge is tested.


---

- finished: 2026-11-10T04:10:14+00:00
- duration: 27.40s
- status: ok
- input_tokens: 4
- output_tokens: 1791
- cache_read_tokens: 36981
- cache_write_tokens: 8256
- cost_usd: 0.109696
