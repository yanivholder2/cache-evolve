# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-10-20T14:39:47+00:00

## timeline

### 14:39:53 thinking



### 14:39:59 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- One-shot inserts displacing entries with proven reuse is the most consistent problem with positive signal. Filtering them at miss time (shared-frequency admission gate, Cycles 4-5) and letting them leave first (eviction-next insert, Cycle 5) both showed hit-rate gains on workload-a.\n- Recurrence gating (ghost list admitting a key only on its second miss, Cycle 7) is the one other positive hit-rate result and fits the same one-shot-insert direction.\n- Scan-resistance through probationary-first eviction (Cycle 3) was the first real hit-rate gain in the ledger; later cohorts that kept this base did not regress from it.\n- Exact-minimum victim selection buys little: bounding the victim scan (Cycles 3-4) left hit-rate flat.\n\n## Recurring failures\n\n- Hit-rate-only geometry and aging tweaks have repeatedly come back flat or marginal (bucket repartitioning, fingerprint filter, aging LFU and its counter-width and insert-count variants, Cycles 1-2). Without a cost dimension they have not paid off.\n- Protected-tier promotion with forced demotion has failed in each attempt (Cycles 4-6). Shrinking the protected region did not change hit-rate, and removing demotion alone did not recover it, so the second-hit or demotion signal is the suspect.\n- Compact or saturating frequency stores (direct-mapped, u8 saturating) lowered hit-rate when changed together, with the cause unresolved (Cycle 6).\n- Tie-admission in the shared-frequency predicate gave no benefit (Cycle 7). Victim-end and front-of-probation placement for newly admitted keys lowered hit-rate (Cycles 6-7).\n- Per-op time and metadata size have not been measured in any cycle. Every cost-motivated claim is therefore untested, and flat or negative hit-rate results cannot be read as a verdict on mechanism.\n\n## Open questions\n\n- Winner selection does not match the hit-rate evidence in Cycles 2, 4, 6 and 7 (and the recorded Winner in Cycle 6 is flat). Which dimension or rule drives it must be settled before any Winner is used as a precedent.\n- Several insights could not confirm that their overlay was active under the harness. Until that is verified, flat results remain ambiguous.\n- Insert position semantics are unclear: does \"eviction-next\" (gain) mean the same thing as \"victim end\" (loss)? A newly admitted key may need some residency before it is eviction-eligible.\n- Does the shared-frequency admission gate stack with eviction-next insert, or are they redundant? Does the ghost-list gain hold on a second workload?\n- Is the protected-tier failure caused by the second-hit signal or by the demotion rule?\n- What are the metadata-size and per-op costs of the shared counter, the ghost list, and the second region?\n\n## Per-policy notes\n\n- **AgedLfu:** Two null or flat results (Cycles 1-2). Whether the halving fires under the harness is unknown. Not pursued further on hit-rate alone.\n- **Fixed-width / bucketed geometry:** Only marginal hit-rate gain (Cycle 1). Cost of wider scans is unmeasured.\n- **Fingerprint filter (bucket):** Hit-rate flat; its per-op benefit is untested (Cycle 2).\n- **Probationary-first / two-region bucket:** The most promising base for scan resistance (Cycle 3). Protected-tier variants have repeatedly failed (Cycles 4-6); shrinking the protected region alone was neutral.\n- **Shared-frequency admission gate:** The clearest positive admission signal (Cycles 4-5). Its metadata cost is unmeasured. Tie-admission variant gave no benefit (Cycle 7).\n- **Ghost list (second-miss admission):** Only positive hit-rate result in Cycle 7; needs a second workload and metadata measurement.\n- **Direct-mapped or saturating frequency counter:** Lost hit-rate; causes not separated (Cycle 6).\n- **Bounded victim scan:** Flat hit-rate in two cycles; a candidate only for per-op savings, once measured.\n"
}
```

### 14:40:02 chat

I wrote `cross-cycle-insights.md` to the output directory for Cycle 0007, with the four required sections in order. I built it only from the inlined ledger digest and did not read any previous version of the file.

The main points it records:

- **Recurring wins:** One-shot-insert filtering is the most consistent positive direction. The shared-frequency admission gate and the eviction-next insert position both showed gains, and the Cycle 0007 ghost list (admit on second miss) fits the same pattern. Probationary-first eviction was the first real hit-rate gain.
- **Recurring failures:** Hit-rate-only geometry and aging tweaks have come back flat. Protected-tier variants have failed in three consecutive cycles. Compact frequency stores and tie-admission lost or gained nothing. Per-op time and metadata size have never been measured.
- **Open questions:** The Winner label disagrees with the hit-rate evidence in several cycles, so it should be checked before any Winner is used as a precedent. Several overlays were never confirmed as active. Insert-position semantics need pinning down, and the cost of the new mechanisms is still unknown.
- **Per-policy notes:** Short notes for AgedLfu, bucket geometry, the fingerprint filter, the two-region bucket, the admission gate, the ghost list, the frequency counter, and the bounded victim scan.

The file has no numbers beyond the cycle references, and it uses qualitative terms for dimensions, as the constraints require.


---

- finished: 2026-10-20T14:40:03+00:00
- duration: 15.79s
- status: ok
- input_tokens: 4
- output_tokens: 2983
- cache_read_tokens: 39364
- cache_write_tokens: 12400
- cost_usd: 0.004366
