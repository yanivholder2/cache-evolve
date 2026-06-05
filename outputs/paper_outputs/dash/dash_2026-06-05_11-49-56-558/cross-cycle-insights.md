# Cross-Cycle Insights

## Recurring wins

- **Hot-tier topology expansion is the highest-leverage structural lever.** Moving the normal/stash split from equal halves toward a large-normal configuration (50/50 → 75/25 → 7/8) produced clear hit-rate gains across multiple cycles; each step confirmed the hot working set was still under-provisioned. The ceiling was located at roughly 7/8 (≈28 normal buckets); gains compound but diminish with each increment.
- **MRU insertion position dominates universally on workload-a.** Every variant that placed newly-inserted or re-admitted items at the recency-hot (Back/MRU) end outperformed those that withheld MRU status. Short reuse gaps mean displaced items reliably get re-accessed before eviction, so probationary or LRU-entry positioning is consistently punishing.
- **Zero-access eviction guard (frequency protect) produces a clear, durable win.** Preferring to evict items with no in-tier access over confirmed-reuse residents closed a real loss site at the probing boundary; the zero-access threshold is precisely right for this workload — single in-tier touches reliably predict membership in the warm working set.
- **Second-chance stash recovery stacks cleanly with the frequency guard.** Unconditionally promoting stash-evicted items to the LRU end of their target bucket addresses a distinct pipeline stage (stash overflow) from the eviction guard (probing selectivity); the two gains are additive with no interference.
- **Stash-exit counter credit closes a residual protection gap.** Crediting a `lfu_counter` access before re-inserting a stash-demoted item eliminates the window in which it is indistinguishable from a cold newcomer under frequency-protect eviction. Gain is modest but genuine, confirming that per-item accounting accuracy matters even at the exit boundary.

## Recurring failures

- **Frequency-based eviction (LFU and all aging variants) is structurally incompatible with workload-a.** Aggressive co-resident decay, threshold-gated decay, and standalone LFU all underperformed position-based recency. The workload's access distribution is warm and recency-dominant; item position encodes a stronger future-access signal than any frequency counter tested.
- **Ghost set re-admission is near-zero across all configurations.** Ghost re-admission was tested at multiple positions (Front, Back) and yielded only noise-level movement every time. The workload does not cycle through the stash boundary with sufficient frequency for bypass tracking to pay off.
- **LRU/Front insertion for newly-arriving or re-promoted items consistently regresses.** Probationary positioning — whether at ghost re-admission, demotion cascades, or probing entry — deprives items of MRU protection during their first in-tier access window, creating eviction blind spots rather than improving selectivity.
- **Timestamp-based promotion is harmful when creation-time pollutes the ordering.** Recording timestamp on insertion allows cold items whose wall-clock creation is newer than a hot item's last access to win promotion, leaking cold items into the normal tier. This interaction destroyed the eviction-guard benefit when both were active.
- **Geometry expansion beyond 7/8 is counterproductive.** Pushing to 15/16 left too few stash slots to buffer cold-arrival bursts; second-chance recovery could not compensate, and the marginal loss from stash shallowness overtook the marginal gain from wider normal capacity.
- **Stash admission weakening always costs more than it saves.** Every attempt to bypass or gate the stash — front-door bypass for cold insertions, LFU-gated promotion — degraded hit-rate by allowing cold or slow-qualifying items to displace warm residents. The stash's role as an admission filter appears fundamental for this workload.

## Open questions

- **Soft-capping `lfu_counter` to prevent staleness.** Long-lived hot items may accumulate artificially high counts that crowd out newly-warm items under frequency-protect eviction. This axis has been flagged repeatedly but never tested; it is the most clearly identified untried lever.
- **Whether stash-exit counter credit interacts with a soft cap.** If both are active, the credit sets a floor while the cap sets a ceiling on counter values; their interaction — cooperative or cancelling — is unknown.
- **Whether timestamp promotion can be rehabilitated.** Recording a timestamp only on the first *hit* (not insertion) would eliminate creation-time contamination; the underlying concept may still be sound if that one failure mode is removed.
- **Whether a second workload trace unlocks blocked axes.** Frequency discrimination, ghost re-admission, and probationary insertion have all been structurally ruled out for workload-a's Zipfian/recency-dominant pattern. A scan-heavy or bursty-cold trace might make these axes genuinely competitive.
- **Intra-normal-tier eviction ordering at non-zero thresholds.** The frequency guard's zero-access cutoff is confirmed optimal; whether a graduated threshold (e.g., guard items with ≥2 hits once the normal tier is very wide) could extract further selectivity at 7/8 geometry has not been tested cleanly.

## Per-policy notes

- **ClassicLRU (pure position-based recency):** Solid baseline throughout; never catastrophically wrong on workload-a. Its recency ordering is the common substrate that all winning variants have preserved or strengthened.
- **ClassicLRUFreqProtect (zero-access eviction guard):** Current champion policy. The zero-access threshold is precisely calibrated to workload-a; single in-tier touches are reliable reuse predictors and must remain protected. Stacks cleanly with second-chance recovery.
- **Second-chance stash recovery:** Real, stable gain when insertion position is LRU (Front). Back/MRU insertion for second-chance items is definitively contraindicated — it grants unproven items immediate promotion candidacy.
- **LFU and aging-LFU variants:** Only marginally competitive on frequency-skewed sub-traces; harmful on the broader workload-a distribution. Aggressive co-resident decay is the most harmful formulation. Should be reserved for workloads with confirmed frequency structure.
- **Ghost-set re-admission:** Consistently noise-level; permanently retired for workload-a in all tested forms (conditional bypass, Front-insert, Back-insert).
- **TimestampLRU:** Replicates some recency-ordering gains already captured by MRU insertion and topology; does not surpass the position-based approach, and is actively harmful when creation-time contaminates the ordering.
- **Stash-exit counter credit:** Small but clean win; eliminates the zeroed-counter vulnerability for stash-demoted items re-entering the normal tier. Should be part of the standing baseline going forward.
