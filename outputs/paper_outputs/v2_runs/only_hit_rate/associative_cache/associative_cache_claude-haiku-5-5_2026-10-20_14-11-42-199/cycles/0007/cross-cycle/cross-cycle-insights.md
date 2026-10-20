## Recurring wins

- One-shot inserts displacing entries with proven reuse is the most consistent problem with positive signal. Filtering them at miss time (shared-frequency admission gate, Cycles 4-5) and letting them leave first (eviction-next insert, Cycle 5) both showed hit-rate gains on workload-a.
- Recurrence gating (ghost list admitting a key only on its second miss, Cycle 7) is the one other positive hit-rate result and fits the same one-shot-insert direction.
- Scan-resistance through probationary-first eviction (Cycle 3) was the first real hit-rate gain in the ledger; later cohorts that kept this base did not regress from it.
- Exact-minimum victim selection buys little: bounding the victim scan (Cycles 3-4) left hit-rate flat.

## Recurring failures

- Hit-rate-only geometry and aging tweaks have repeatedly come back flat or marginal (bucket repartitioning, fingerprint filter, aging LFU and its counter-width and insert-count variants, Cycles 1-2). Without a cost dimension they have not paid off.
- Protected-tier promotion with forced demotion has failed in each attempt (Cycles 4-6). Shrinking the protected region did not change hit-rate, and removing demotion alone did not recover it, so the second-hit or demotion signal is the suspect.
- Compact or saturating frequency stores (direct-mapped, u8 saturating) lowered hit-rate when changed together, with the cause unresolved (Cycle 6).
- Tie-admission in the shared-frequency predicate gave no benefit (Cycle 7). Victim-end and front-of-probation placement for newly admitted keys lowered hit-rate (Cycles 6-7).
- Per-op time and metadata size have not been measured in any cycle. Every cost-motivated claim is therefore untested, and flat or negative hit-rate results cannot be read as a verdict on mechanism.

## Open questions

- Winner selection does not match the hit-rate evidence in Cycles 2, 4, 6 and 7 (and the recorded Winner in Cycle 6 is flat). Which dimension or rule drives it must be settled before any Winner is used as a precedent.
- Several insights could not confirm that their overlay was active under the harness. Until that is verified, flat results remain ambiguous.
- Insert position semantics are unclear: does "eviction-next" (gain) mean the same thing as "victim end" (loss)? A newly admitted key may need some residency before it is eviction-eligible.
- Does the shared-frequency admission gate stack with eviction-next insert, or are they redundant? Does the ghost-list gain hold on a second workload?
- Is the protected-tier failure caused by the second-hit signal or by the demotion rule?
- What are the metadata-size and per-op costs of the shared counter, the ghost list, and the second region?

## Per-policy notes

- **AgedLfu:** Two null or flat results (Cycles 1-2). Whether the halving fires under the harness is unknown. Not pursued further on hit-rate alone.
- **Fixed-width / bucketed geometry:** Only marginal hit-rate gain (Cycle 1). Cost of wider scans is unmeasured.
- **Fingerprint filter (bucket):** Hit-rate flat; its per-op benefit is untested (Cycle 2).
- **Probationary-first / two-region bucket:** The most promising base for scan resistance (Cycle 3). Protected-tier variants have repeatedly failed (Cycles 4-6); shrinking the protected region alone was neutral.
- **Shared-frequency admission gate:** The clearest positive admission signal (Cycles 4-5). Its metadata cost is unmeasured. Tie-admission variant gave no benefit (Cycle 7).
- **Ghost list (second-miss admission):** Only positive hit-rate result in Cycle 7; needs a second workload and metadata measurement.
- **Direct-mapped or saturating frequency counter:** Lost hit-rate; causes not separated (Cycle 6).
- **Bounded victim scan:** Flat hit-rate in two cycles; a candidate only for per-op savings, once measured.
