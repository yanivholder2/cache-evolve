# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-10-21T14:49:15+00:00

## timeline

### 14:49:30 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- **Reward proven reuse, keep entry cheap and unprotected.** Every cycle points the same way: a ghost hit is enough evidence on its own, and front-of-main / head insertion at freq 0 is a local optimum. Variants that kept that entry point and widened the evidence only slightly made small gains: ghosting main victims, and a ring with a slightly wider effective window.\n- **A bigger main queue via a smaller small queue.** Shrinking the partition (10% → 5% → 2.5%) has won three times, each time by a smaller margin. The gain tracks main capacity, and probably the ghost reach that grows with it.\n- **Eviction order was the last real lever.** The SIEVE hand (survivors stay in place) gave the first gain beyond noise after the FIFO-era knobs stopped helping. Clear-on-pass (one visited bit) improved on it.\n- **Cost-only rewrites keep policy parity.** The indexed slab, fingerprint ring, identity hasher, 32-bit fingerprints and cheap mixer were each checked against a reference with full-state parity. Policy and representation are cleanly separable and compose.\n\n## Recurring failures\n\n- **More generous ghost re-admission.** Credits, a free second lap, a larger or separately budgeted ghost, and small-queue probation all lost. Every ghost hit bypasses the small filter and forces a main eviction, so extra bypass causes churn among proven residents.\n- **Stricter or less protected re-admission.** Tail insertion, near-hand insertion and a halved ghost ring also lost. Re-admits need about one full sweep, and the ghost needs about main_size entries.\n- **Tuning retention strength away from the local optimum.** Under FIFO, raising the cap past 7 stayed flat and decay hurt. Under SIEVE, extra laps, behind-hand newcomer protection and resetting graduate credit all hurt. The best setting depends on the regime and does not carry over between them.\n- **Graduation-threshold changes.** These were flat or negative under both regimes, because the ghost already recovers keys that were hit once in small.\n- **Adaptive split driven by ghost source.** It is biased, because small rejects dominate the ghost, and it reacts too late.\n- **Ignoring the measurement gap.** Per-op-time, metadata-size and a second trace or cache size have been requested in every Continuation and are still missing after ten cycles. Every cost-side change has unproven value, and every gain is fitted to one trace at one size.\n\n## Open questions\n\n- Do the accumulated choices (small split, SIEVE with clear-on-pass, ghosted main victims, main-sized fingerprint ring) generalise to other traces and cache sizes?\n- What do the indexed, ring, hasher and fingerprint changes actually save in per-op-time and metadata-size? Do SIEVE pointer-chasing and double hashing cancel the savings out?\n- Does the split curve keep rising below 2.5%, and does the recent gain come from main space or from ghost reach? Testing that needs a ghost budget held fixed while main grows.\n- Would changing which keys get ghosted (only main victims vs only small rejects), without changing ghost size or placement, matter? Expect a small effect at most.\n\n## Per-policy notes\n\n- **S3-FIFO with FIFO reinsertion in main (cycles 1–6):** 5% split, cap 7 with linear decrement, re-admission at front of main with freq 0, and a shared capped ghost that also holds main victims. All knobs reached a plateau, and only near-noise effects were left on workload-a.\n- **S3-FIFO with SIEVE main (cycles 7–10):** survivors stay in place and one visited bit is cleared on each pass. Graduates carry their small-queue credit, and that one carried lap is optimal. Re-admits enter at the head with freq 0, and the ghost ring is sized to main. A 2.5% split is the current baseline. Retention levers settled under FIFO have to be re-tuned under SIEVE rather than carried over. Under SIEVE, less retention has won so far.\n"
}
```

### 14:49:36 chat

I wrote `cross-cycle-insights.md` in the cycle 0010 output dir, built from scratch from the ten-cycle ledger with the four required sections in order. The main conclusions across all cycles:

- **The policy is close to saturated on workload-a.** Almost every setting (partition split, retention strength, graduation threshold, ghost size and where re-admitted keys are inserted) has reached a local optimum on this one trace. Pushing any of them further in either direction has been flat or slightly worse.
- **The lasting wins are few.** Shrinking the small queue (10% → 5% → 2.5%) won three times, by smaller margins each time. Switching main to a SIEVE-style hand, then simplifying it to one visited bit, gave the only gains clearly above noise in the later cycles.
- **Cost-side changes are policy-safe but unproven.** Each one matched the reference simulation exactly, so none changed hit-rate, but nobody has measured whether they are actually cheaper.
- **The measurement gap is the biggest unresolved issue.** Per-op-time, metadata-size and a second trace or cache size have been requested in every cycle's Continuation and are still missing after ten cycles. Every gain may be specific to workload-a at one cache size.
- **Per-policy notes are split by eviction regime.** Retention settings that were optimal with FIFO reinsertion (cycles 1–6) did not carry over to SIEVE (cycles 7–10), where less retention has won so far.


---

- finished: 2026-10-21T14:49:37+00:00
- duration: 22.07s
- status: ok
- input_tokens: 4
- output_tokens: 2005
- cache_read_tokens: 42081
- cache_write_tokens: 14928
- cost_usd: 0.167956
