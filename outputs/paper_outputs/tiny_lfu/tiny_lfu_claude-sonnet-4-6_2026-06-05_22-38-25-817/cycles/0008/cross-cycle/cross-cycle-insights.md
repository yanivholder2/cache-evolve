# Cross-Cycle Insights

## Recurring wins

- **Structural eviction-ordering changes outperform admission-policy changes.** Every lasting gain came from reshaping how items are ordered or protected after admission (SLRU segmentation, victim-scan, sketch-quality). Admission-side interventions consistently backfired or were neutral.
- **Frequency-estimation quality is a compounding lever.** Conservative CMS update (cycle 0006) and doubled sketch width (cycle 0007) both yielded real hit-rate gains by sharpening the signal that all downstream policy decisions depend on. The two mechanisms addressed distinct error sources and their gains stacked.
- **k=4 victim scan on probationary is durable.** Sampling the four LRU-tail candidates and evicting the minimum-frequency one has survived across multiple baseline transitions without regression. Its benefit is bounded but consistent.
- **PROMO_THRESHOLD + `was_protected` bypass is a safe policy pair.** The threshold prevents borderline-frequency newcomers from consuming protected headroom; the bypass protects genuinely hot items caught in capacity-overflow demotions. Together they interact cleanly with the victim scan.

## Recurring failures

- **Sketch aging (count halving) is consistently harmful.** At every cadence tested — short (cycle 0001), long (cycle 0002), very slow (cycle 0008) — periodic resets erase more valid frequency signal than stale-count noise they remove. The workload's frequency distribution is stable; accumulated counts are signal, not artifact. This axis is closed.
- **Ghost-set re-admission degrades on Zipfian.** Fast-tracking recently evicted items back into protected re-promotes the items that already lost the TinyLFU admission contest — they are genuinely cold. Regressed in both direct tests; the failure mode is well-understood and the axis is closed.
- **SLRU split-ratio tuning is exhausted past 90/10.** The gain trajectory from 80/20 → 90/10 did not continue to 95/5; the hot working set fits within the existing protected budget and additional headroom rescues nothing.
- **Protected-segment victim scanning adds nothing.** The PROMO_THRESHOLD gate pre-filters entrants, leaving the protected LRU tail frequency-homogeneous; a scan rarely finds a colder candidate than the default LRU head. Tested and closed in cycle 0006.
- **Admission-window / newcomer capacity reservation is immediately punishing.** Reserving permanent capacity for a window segment shrank the main LRU before any newcomer-grace benefit materialized. This trace does not produce items that accumulate frequency fast enough inside the window to recoup the structural cost.
- **Direct-to-protected bypass of probationary causes premature churn.** Skipping the probationary staging area triggers demotion cascades before demand is reconfirmed by a real access; probationary is a load-bearing timing gate, not mere overhead.

## Open questions

- **Do policy closures from cycle 0005 hold under the now-sharper sketch?** PROMO_THRESHOLD gating and the victim-scan window were both closed when collision noise was substantially higher. Re-probing them against the conservative-update + wide-sketch baseline is the highest-priority remaining experiment.
- **Is the remaining hit-rate gap attributable to irreducible workload entropy, or does a structural change still have headroom?** Quantifying how much of the gap is unpredictable cold-item noise versus addressable policy error would clarify whether further investment is worthwhile.
- **Does frequency-ordered eviction within the protected segment hold any remaining value?** It was explored only once (cycle 0006) and found flat; the conclusion may be different under sharper sketch estimates.
- **Are the present conclusions portable beyond this Zipfian trace and cache-size regime?** All closed axes and confirmed wins were measured on a single workload and capacity; transferability to non-Zipfian or differently sized configurations is unverified.

## Per-policy notes

- **SLRU:** 90/10 split is settled as optimal for this workload. Protected segment's internal LRU tail is frequency-homogeneous after PROMO_THRESHOLD gating, making in-segment victim selection pointless. Probationary functions as a timing/reconfirmation gate and must not be bypassed.
- **TinyLFU / CountMinSketch:** Conservative update and 8× width are both fixed baseline components; they address complementary error sources (estimate inflation vs. row-collision independence). Depth, aging, and hash-seed diversity are all confirmed closed.
- **Victim selection:** k=4 scan on probationary is effective and saturated — widening to k=8 changed no decisions, so the coldest reachable candidate already appears within the first four LRU-tail positions. Mirroring the scan to protected is closed.
- **Admission gate:** The existing strict-inequality TinyLFU frequency gate handles one-hit-wonders well. Raising the admission floor to frequency ≥ 2 was redundant on a warm Zipfian trace. Ghost-set and direct-to-protected bypasses are both closed with understood failure modes.
