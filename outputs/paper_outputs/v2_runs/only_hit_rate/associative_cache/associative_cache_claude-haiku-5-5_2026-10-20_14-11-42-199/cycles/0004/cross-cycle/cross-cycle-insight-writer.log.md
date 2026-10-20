# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-10-20T14:27:35+00:00

## timeline

### 14:27:46 thinking



### 14:27:51 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Hit-rate has moved positively only when a change alters which entries get displaced (scan-resistant probationary-first eviction in Cycle 0003, shared-frequency admission gate in Cycle 0004), not when it tweaks how the existing ranking is computed.\n- Capping or sampling the exact-minimum victim scan left hit-rate flat in two cycles (Cycles 0003 and 0004). This is a plausible cost-side saving, but it is unmeasured.\n- Putting incoming and resident keys on one frequency scale appears to fix the admission-gate loss seen in Cycle 0003. One cycle of evidence so far.\n\n## Recurring failures\n\n- Cost-motivated changes have been judged on hit-rate alone in every cycle (1–4). Per-op time and metadata size were never recorded, so no cost claim has been tested. This is the most persistent gap.\n- Aging-based LFU variants (periodic halving, saturating counters, insert-count triggers) produced no measurable hit-rate gain in Cycles 1 and 2. Whether the evaluation even selects or fires the aging policy is unverified.\n- Stacking a second reuse tier on probationary-first eviction lowered hit-rate (Cycle 4). The failure looks tied to the protection cap and demote-on-promotion path rather than the idea of a second tier.\n- Admission gating without a common frequency scale lowered hit-rate (Cycle 3). Treat that as a flawed comparison, not a verdict on admission.\n- Hit-rate-flat results cannot be read as nulls until overlay activation is confirmed. Cycles 3 and 4 both had variants whose overlays could not be shown to run.\n\n## Open questions\n\n- Which dimension made the Winner in Cycles 2 and 4? In both, the labeled Winner had no hit-rate gain, while another variant did (Cycle 4). Is the Winner rule using a metric missing from the insights?\n- Is the shared-frequency admission gain reproducible beyond workload-a, and what metadata or per-op cost does its shared counter add?\n- Do the cost-side savings claimed for the fingerprint filter, the saturating u8 counter, the static-dispatch refactor, and the wider buckets show up once per-op time and metadata size are measured?\n- Does the AgedLfu halving fire at all under the harness, and how many halvings occur per run?\n- Does a protected tier work if the cap or demotion rule changes, rather than the promotion path alone?\n\n## Per-policy notes\n\n- Probationary-first / two-region eviction: positive in Cycle 3; a stacked second-hit protected tier was negative in Cycle 4. The base idea is the strongest hit-rate lead so far.\n- Bounded or sampled victim scan: hit-rate flat in Cycles 3 and 4. Only a cost measurement can justify it.\n- Frequency-gated admission: negative when the two estimates were on different scales (Cycle 3); positive with a shared counter (Cycle 4).\n- AgedLfu and its variants (periodic halving, u8 saturating counters, insert-count trigger): no measurable hit-rate gain across Cycles 1 and 2. Activation is unverified.\n- Bucket geometry: fixed-width buckets (Cycle 1) gave a marginal hit-rate gain with unmeasured scan cost. Bucket-conflict misses look like a small share of total misses at this cache size, which caps geometry gains.\n- Static dispatch on the bucket policy (Cycle 1): hit-rate neutral by design; latency effect unmeasured.\n- Per-bucket fingerprint filter (Cycle 2): hit-rate flat; the Winner label is unsupported by the cited evidence. Keep only with a per-op-time measurement.\n"
}
```

### 14:27:54 chat

I wrote `cross-cycle-insights.md` to the cross-cycle output directory for cycle 0004. It has the four required sections in order, built only from the ledger digest. I did not read the previous file, and the write succeeded without an overwrite error.

The main signals:

- **Recurring win:** Hit-rate has only moved positively when a change alters which entries get displaced (probationary-first eviction in Cycle 3, the shared-frequency admission gate in Cycle 4). Capping the victim scan left hit-rate flat twice, which points to a possible cost saving that nobody has measured yet.
- **Recurring failure:** Every cycle has judged cost-motivated changes on hit-rate alone, so no per-op time or metadata claim has been tested. Aging-based LFU variants have shown no hit-rate gain across Cycles 1 and 2.
- **Open threads:** Two cycles labeled a Winner whose cited evidence showed no hit-rate gain, so the Winner rule needs checking before any of those labels are treated as precedent. Overlay activation is also unconfirmed for several variants.


---

- finished: 2026-10-20T14:27:54+00:00
- duration: 19.22s
- status: ok
- input_tokens: 4
- output_tokens: 3643
- cache_read_tokens: 36655
- cache_write_tokens: 10481
- cost_usd: 0.004285
