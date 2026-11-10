# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-11-10T10:06:48+00:00

## timeline

### 10:07:02 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- Per-key evidence beats global signals. Gating on what each key carries with it (its small-queue freq when it was ghosted, then its ghost age) gave the only clean policy gains in the lineage (cycles 0004–0005). Every global proxy failed: main occupancy, ghost-return rate, and the unweighted ARC split.\n- The strict two-reuse admission into main, plus the freq reset, has held up as the base in every cycle. Loosening it has never survived steady-state pressure.\n- Ghost age works as a floor for freq-0 returnees, and about one small-queue turnover is the right boundary. A quick return marks hm_0's real reuse; a late return marks web_0's one-off rereferences.\n- Mechanisms whose windows scale with `small_size` can move cells at the large cache size. Fixed or global knobs only ever moved the small size.\n- Cost-only refactors (the O(1) index/slab, fast hasher, compact `Loc`) have stayed bit-identical in hit-rate every time. They are safe to port onto any policy parent.\n\n## Recurring failures\n\n- Single-constant tweaks to the threshold or to the static small/main split. hm_0 and web_0 split in opposite directions, and the effect flips sign across cache sizes (cycles 0001–0002).\n- Rewarding ghost returnees through global signals or flat credit, such as rate-switched thresholds or second-chance credit. The ghost mostly holds weak keys, and main is always full, so any extra protection evicts entries that would have been hit (cycles 0003, 0006).\n- Relap stacking. Resetting freq erased the ghost signal (cycle 0005). Keeping freq 1 made the relap redundant with the ghost path and cost small-queue capacity (cycle 0006).\n- Re-admitting keys to small at the small cache size. Displacing residents keeps costing more than the main pollution it avoids.\n- Loosening the age window beyond one turnover, or adding age ceilings on freq ≥ 1 returnees. Both give back hit-rate on web_0.\n- Measurement gap: per-op-time and metadata-size have been missing from the fitness vector for six cycles in a row. Hypotheses that assumed these were reported were evaluated on hit-rate alone.\n\n## Open questions\n\n- What limits web_0 at the large size? Every ghost and small routing change has left it flat, whether it tightened or loosened admission. Candidates are main's eviction order, the split at that size, and first-admission filtering of never-seen keys.\n- Would a window narrower than one turnover sharpen the hm_0 / web_0 split?\n- How much do the pending cost-only variants actually save in per-op-time and metadata-size, once those are measured? Alignment may cancel the layout savings on 8-byte keys.\n- Is a direct comparison of what promoting a key gains against what displacing a main entry costs workable as a gate?\n\n## Per-policy notes\n\n- S3-FIFO (the only policy in this lineage): the current best is cycle 0005's v002, the age-gated ghost path (freq-gated, age window at 1×) on the O(1) index. Ghost and small routing looks locally optimal and close to exhausted after cycle 0006. Gains have been small and mostly at the small cache size.\n- hm_0 is lenient-friendly at the small size only; its earlier preference for a looser threshold was largely a cold-start effect. web_0 punishes any loosening of admission, and at the large size it has not moved under any change so far.\n- Next directions: target main's eviction and first-admission handling, and port the cost changes onto the policy parent. Push for the cost dimensions to be measured before spending more hypotheses on cost.\n"
}
```

### 10:07:10 thinking



### 10:07:10 chat

I've written `cross-cycle-insights.md` in the cycle 0006 cross-cycle directory. It uses the four required sections in order and draws only on the six-cycle ledger digest.

- **Recurring wins:** Admission and ghost-routing decisions based on each key's own evidence (its freq when ghosted, then its ghost age) gave the only clean policy gains. Every global signal failed. The strict two-reuse rule has held up as the base in every cycle. A ghost-age floor of about one small-queue turnover is the right boundary.
- **Recurring failures:** Single-constant tweaks to the threshold or split, ghost-rate switching, flat credit for ghost returnees, relap stacking, and age windows wider than one turnover all lost hit-rate. Per-op-time and metadata-size have been missing from the evaluation for six cycles in a row.
- **Open questions:** What limits web_0 at the large cache size, which no ghost or small-queue routing change has moved. Whether a narrower age window helps. What the cost-only variants actually save once those two dimensions are measured.
- **Per-policy notes:** S3-FIFO is the only policy in this lineage. The current best is cycle 0005's age-gated ghost path. The ghost and small-queue routing axis looks exhausted, so the suggested next targets are main's eviction order and how never-seen keys are first admitted.


---

- finished: 2026-11-10T10:07:10+00:00
- duration: 22.63s
- status: ok
- input_tokens: 4
- output_tokens: 1839
- cache_read_tokens: 39887
- cache_write_tokens: 11305
- cost_usd: 0.135918
