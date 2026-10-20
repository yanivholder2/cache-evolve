# Cross-Cycle Insights

## Recurring wins

- **Eviction-side frequency filtering (probationary victim scan)** — scanning a small window (k=4) of the probationary LRU tail and evicting the minimum-frequency candidate has been the single most robust structural improvement, first appearing in cycle 0004 and surviving all subsequent lineages.
- **Sketch-quality improvements stack with policy changes** — conservative CMS update (cycle 0006) and doubled sketch width (cycle 0007) each produced independent gains, and both amplified the effectiveness of upstream policy knobs by reducing estimation noise that had been masking correct decisions.
- **Policy recalibration after sketch improvement pays off** — PROMO_THRESHOLD had been set against a noisy sketch baseline; revisiting it after sketch quality was improved (cycle 0009) unlocked a real though small gain, confirming that policy and estimation quality should be co-tuned.
- **SLRU 90/10 split with unconditional-or-gated promotion** is a stable structural foundation; every cycle's winner has operated within this framework once it was established in cycles 0001–0002.

## Recurring failures

- **Sketch aging / periodic count halving** — harmful at every cadence tested (cycles 0001, 0008). The workload's frequency distribution is stable across the replay window; accumulated counts are signal, and any reset discards more valid signal than stale noise.
- **Ghost-set re-admission** — regressed twice (cycles 0003, 0004). Ghost-ring entries are precisely the items that lost the TinyLFU admission contest; fast-tracking them displaces legitimately hot residents on a Zipfian workload where cold items stay cold.
- **Protected-segment victim scanning** — tested repeatedly (cycles 0005, 0009, 0010) under both noisy and accurate sketch baselines. Protected's PROMO_THRESHOLD entry gate leaves the LRU tail with a narrow frequency spread; the scan finds no colder candidate than the default LRU head.
- **Admission-window / capacity reservation** — reserving a permanent buffer segment (cycle 0001) shrank the main LRU and was punished immediately; newcomer grace benefits never materialized on this trace.
- **Put-path promotion symmetry** — structurally sound but irrelevant on a read-dominant Zipfian trace (cycle 0004); hot items accumulate frequency via get() alone.
- **SLRU split-ratio changes** — exhausted in both directions. Enlarging protected (cycles 0002–0005) and shrinking it (cycle 0010) both failed to improve hit rate; the hot working set fits comfortably within 10% protected at this cache size.

## Open questions

- **Has the design reached an irreducible ceiling for this trace?** Cycles 0009–0010 produced no net advancement and every identified structural axis is closed. The remaining gap may belong to workload entropy (cold-start misses on the Zipfian long tail) rather than policy.
- **Would a pre-admission cold-item filter** (e.g., a Bloom-filter-style guard at the probationary entry point) address any portion of the remaining gap, or is the gap entirely workload-driven?
- **Are the current conclusions trace-specific?** Conservative-update gains, ghost-set failures, and the PROMO_THRESHOLD floor are all calibrated against one Zipfian replay; robustness across non-Zipfian or bursty workloads is unverified.
- **Architectural pivot question**: given that all parametric and structural tuning knobs are exhausted, would an entirely different base eviction policy (e.g., W-TinyLFU, CLOCK-Pro) move the ceiling rather than re-probe it?

## Per-policy notes

- **TinyLFU admission gate**: strict inequality (freq > 0) is already well-calibrated; raising the floor to 2 was redundant on a warm trace (cycle 0006). The gate handles one-hit-wonders effectively and does not need restructuring.
- **SLRU segmentation**: 90/10 protected/probationary is the stable optimum for this workload-size pairing. Both split directions are closed. The probationary segment is a pass-through staging area, not a contention zone.
- **PROMO_THRESHOLD gate**: optimal at 6 with the current accurate sketch. The 8→6 reduction (cycle 0009) was enabled by sharper estimates; 6→5 (cycle 0010) triggered protected-segment churn that outweighed the admission benefit. Floor is confirmed at 6.
- **`was_protected` bypass**: essential partner to aggressive gating — allows overflow-demoted hot items to re-enter protected without re-earning the threshold, preventing capacity-churn from stranding genuinely hot items.
- **CountMinSketch**: fully tuned. Conservative update (cycle 0006), 8× width (cycle 0007), standard depth (cycle 0008), and diverse hash seeds (cycle 0007) are all resolved. No further sketch changes expected to pay off on this workload.
- **Probationary victim scan (k=4)**: closed as saturated. The coldest reachable candidate appears within the first few LRU-tail positions; widening the window changes no eviction decisions. Scan window axis is confirmed closed at k=4.
