# Cross-Cycle Insights

## Recurring wins

- **Eviction/resident-ordering changes consistently outperform admission-side changes.** Every structural win came from reshaping how items are ordered or selected for eviction after admission (SLRU segmentation, k=4 victim scan, promotion gating), not from adjusting who gets in.
- **A large protected segment (90/10 split) reliably outperforms smaller ratios.** The hot working set fits comfortably in 90 % of capacity; shrinking probationary to 10 % imposed no measurable cost on this Zipfian trace.
- **The `was_protected` bypass is the essential safety valve for aggressive promotion gating.** Overflow-demoted items would otherwise be starved back into cold storage, so the bypass prevents the gate from penalizing legitimately hot items caught by capacity churn.
- **Victim-side frequency scanning (k=4 LRU tail) rescued warm survivors that strict LRU displaced**, opening a new positive axis after promotion and ratio tuning had both saturated.

## Recurring failures

- **Sketch aging (counter halving) is consistently harmful regardless of cadence.** Tested at two very different reset intervals (cycles 0001 and 0002); both degraded hit rate. The trace's frequency distribution is stable across the replay window — accumulated counts are signal, not noise, and resetting them destroys useful history.
- **Ghost-set re-admission regressed twice (cycles 0003 and 0004).** Items evicted via TinyLFU are genuinely cold; fast-tracking them back into the protected segment displaces legitimately hot residents. The failure mechanism is well understood and the idea is closed.
- **Admission-window reservation (newcomer segment) failed immediately.** Permanent buffer space for new arrivals cannibalized the main LRU before any grace-period benefit could materialise; the trace does not produce items that accumulate frequency fast enough to recoup the structural penalty.
- **All parametric axes are now saturated.** Split-ratio (80/20 → 90/10 → 95/5), scan window (k=4 → k=8), promotion threshold values, and put-path symmetry all converged to zero additional gain by cycle 0005. Five cycles of tuning have reached a local optimum within the current structural design.

## Open questions

- **Frequency-ordered eviction within the protected segment** (replacing its internal LRU) is the most promising unexplored axis — cold tails of the protected set may be evictable without LRU's recency bias.
- **TinyLFU admission gate tuning** (sketch size, frequency threshold, counter initialization) has not been directly probed; it may be the remaining structural frontier given that post-admission segment management is nearly exhausted.
- **Interaction between the k=4 victim scan and PROMO_THRESHOLD gating** is unresolved — the two were tested in combination and are compatible, but their individual contributions were never isolated.
- **Cache-size sensitivity** — whether marginal wins hold at different capacity-to-working-set ratios — remains untested and could reopen closed axes at different operating points.

## Per-policy notes

- **TinyLFU + SLRU (primary policy):** Five cycles of refinement. Current best configuration: 90/10 split, k=4 probationary victim scan with frequency tie-breaking, PROMO_THRESHOLD≥8, and `was_protected` demotion bypass. All five knobs should be treated as fixed; further parametric adjustment is not warranted.
- **Admission gate (TinyLFU sketch):** Has performed well throughout — handles one-hit-wonders effectively without modification. No admission-side change has improved on it, but direct tuning of the sketch itself has not been attempted.
- **Protected-segment internals:** LRU ordering within protected has been preserved across all cycles. Frequency-aware ordering within this segment is the only structural sub-component that remains completely unexplored.
- **Probationary segment:** Behaves as a pass-through staging area on this workload rather than a contention zone. Reducing it to 10 % caused no harm; its primary role is absorbing the k=4 victim scan, not holding contested items.
