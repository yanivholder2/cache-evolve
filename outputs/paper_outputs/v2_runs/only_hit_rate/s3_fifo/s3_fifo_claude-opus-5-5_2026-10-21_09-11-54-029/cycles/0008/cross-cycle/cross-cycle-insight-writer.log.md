# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-10-21T09:50:49+00:00

## timeline

### 09:51:04 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- On workload-a, rewarding keys that have shown repeated reuse beats making entry easier. This has held across nearly every policy cycle (0001–0006, 0008).\n- Main-queue eviction order (cycles 0007–0008) has been the only lever to give gains beyond noise since the early partition sweep. SIEVE-style in-place survivors won first. Clear-on-pass, a single visited lap, then improved on that.\n- Shrinking the small queue (to 10%, then 5%) gave small but consistent gains. They came from a larger main queue, not a larger ghost. The curve was close to flat by 5%.\n- Ghost re-admission works best at front-of-main with zero credit. A bare ghost hit is enough evidence of reuse. Ghosting main victims in the shared, capped budget added a sliver.\n- Cost-only representation changes have been policy-neutral every time and compose cleanly: the indexed slab, the fingerprint ring, the identity hasher and 32-bit fingerprints. Each was verified against a reference.\n\n## Recurring failures\n\n- Making re-admission more generous (freq credit, a decoupled or larger ghost, a separate main-victim ghost budget) keeps losing. Each ghost hit bypasses the small filter and forces a main eviction.\n- Making re-admission stricter also loses: tail insertion and small-queue probation. The small window is shorter than the ghost's reuse horizon, so these reject keys the ghost correctly identified.\n- Earlier graduation from small loses or stays flat both with a frequency reset and without one. The ghost already recovers keys that were hit once.\n- Protecting newcomers longer loses under both FIFO and SIEVE. Under SIEVE, behind-hand insertion regressed.\n- Adaptive splits driven by which source a ghost hit came from regressed. The signal is biased toward small rejects and arrives too late.\n- Measuring only hit-rate has been the constant process failure. Per-op-time and metadata-size have gone unmeasured for eight cycles, and only one trace and one size have ever been run. No cost-side change has been judged on its actual purpose.\n\n## Open questions\n\n- Under SIEVE, does a small main-only cap (2–3 laps) beat a single visited bit?\n- With clear-on-pass, should graduation from small use a counter separate from main retention?\n- Should ghost re-admissions get less protection than graduates under SIEVE, for example insertion nearer the hand?\n- What do the indexed, ring, hasher and fingerprint changes actually save in per-op-time and metadata-size? Does SIEVE's pointer-chasing cancel those savings out?\n- Do the accumulated choices (5% split, front-of-main re-admission, ghosted main victims, SIEVE with clear-on-pass) hold on other traces and cache sizes?\n\n## Per-policy notes\n\n- S3-FIFO, FIFO-reinsertion main (cycles 0001–0006): a frequency cap of 7 was best. Going 3→7 helped, 15 was flat, and decay hurt. The partition, graduation, re-admission and ghost-capacity knobs are all settled, flat or negative. The family looked saturated on workload-a.\n- S3-FIFO, SIEVE main (cycles 0007–0008): retention conclusions from the FIFO era do not carry over. Less retention is better, and clear-on-pass beat multi-lap credit. Quickly demoting newcomers near the hand does useful filtering. The gain comes from survivors staying in place. Retention, graduation and re-admission protection under SIEVE are the live policy space.\n- Ghost representation: the fingerprint ring with 32-bit fingerprints and identity hashing is the current form. It is policy-neutral, and false-hit risk is negligible at this size.\n"
}
```

### 09:51:11 chat

I've written `cross-cycle-insights.md` in the cycle 0008 cross-cycle folder, using the four required sections in order and drawing only on the ledger for cycles 0001–0008.

The main threads it carries forward:

- **What keeps working:** keeping keys that have proven reuse beats making it easier for new keys to get in. Changing how the main queue picks what to evict (the SIEVE-style hand from cycle 0007, then a single visited bit from cycle 0008) is the only change to clearly beat noise in a long while.
- **What keeps failing:** ghost re-admission lost whether it was made more generous or stricter. Earlier promotion from the small queue, protecting new keys for longer, and an adaptive small/main split all lost or stayed flat too.
- **The biggest gap:** speed per operation and memory use for cache bookkeeping have gone unmeasured for all eight cycles. Only one trace at one cache size has ever been tested. So none of the speed or memory changes has been judged on what it was meant to improve, and nobody knows whether the hit-rate gains carry over to other traces or sizes.
- **Per-policy split:** the file treats the original requeue-to-head main queue and the SIEVE main queue as separate eras. How long to keep keys behaved differently in each: under requeue-to-head a cap of 7 was best, while under SIEVE keeping keys for less time wins. So the earlier cap and decay results shouldn't be reused for SIEVE.

The file names the open questions for upcoming cycles. They cover how long SIEVE should keep keys, whether promotion from the small queue needs its own counter, how much protection re-admitted keys should get, and the missing measurements.


---

- finished: 2026-10-21T09:51:11+00:00
- duration: 21.65s
- status: ok
- input_tokens: 4
- output_tokens: 1956
- cache_read_tokens: 40189
- cache_write_tokens: 12920
- cost_usd: 0.150534
