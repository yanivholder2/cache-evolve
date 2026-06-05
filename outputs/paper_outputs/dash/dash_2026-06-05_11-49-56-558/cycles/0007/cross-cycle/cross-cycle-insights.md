# Cross-Cycle Insights

## Recurring wins

- **Hot-tier topology expansion** was the earliest and highest-leverage gain: enlarging the normal-bucket pool at the expense of stash produced clear wins in Cycles 1–2, peaking at a 24-normal / 8-stash split in Cycle 3. Once the working set fits comfortably in the hot tier, capacity is no longer the binding constraint.
- **MRU insertion position universally wins on workload-a.** Every attempt to withhold MRU status — at stash re-entry, ghost re-admission, demotion cascades, or probing entry — regressed or was noise. Short reuse gaps mean recently-positioned items are reliably re-accessed before eviction.
- **Eviction selectivity within a tier is a live lever.** Preferring to evict zero-access newcomers over confirmed-reuse residents (Cycle 6 frequency guard) produced the largest single-cycle gain in the late search, confirming the probing boundary was a meaningful loss site even after topology and insertion position were optimised.
- **Orthogonal mechanisms stack cleanly.** Topology + MRU re-entry (Cycles 1–3), frequency-guard + second-chance recovery (Cycle 7): when two variants target genuinely distinct pipeline stages, combining them closes both loss sites without interference and produces additive gains.
- **Stash serves as a load-bearing admission filter.** Respecting stash traversal — never bypassing or weakening it for cold insertions — has been a consistent constraint; the stash gate's quality function outweighs any cost savings from shortcutting it.

## Recurring failures

- **Frequency-based eviction (LFU, aging-LFU, threshold-gated LFU) consistently underperforms.** In every formulation tested (Cycles 1–3, 7), frequency discrimination either matched or degraded ClassicLRU. Workload-a's Zipfian-but-warm access distribution makes item position a stronger future-access signal than any counter variant; even a single in-tier hit is a reliable reuse predictor that aggressive frequency pruning destroys.
- **Ghost/victim-cache re-admission is noise-level regardless of configuration.** Tested in multiple cycles (4–6) with varying position and trigger logic; the workload's reuse pattern does not cycle through the stash boundary frequently enough to benefit.
- **Bypassing or weakening the stash gate regresses.** Front-door cold bypass (Cycle 4 v003) and promotion-threshold gating (Cycle 4 v001) both hurt hit-rate by allowing unqualified items to displace warm residents.
- **Timestamp-based promotion interacts destructively with frequency-guard eviction.** In Cycle 7, using `last_accessed` rather than LRU position to select promotion candidates allowed cold items with newer creation timestamps to win promotion, cancelling the eviction-guard benefit.
- **Raising the frequency-guard threshold beyond zero is harmful.** A single in-tier access is a sufficient warm signal on workload-a; treating single-touch items as eviction candidates shrinks the protected set too aggressively (Cycle 7).
- **Probationary / LRU-entry insertion is consistently punishing.** Segmented-LRU asymmetry and probationary-zone probes (Cycles 3, 5) both regressed — workload-a's access distribution is warm throughout, so demotion buffer positions accelerate churn rather than improve selectivity.

## Open questions

- With frequency-guard eviction and second-chance stash recovery both active, what loss site remains? Stash geometry fine-tuning (e.g., sub-8 stash capacity) and `lfu_counter` staleness management (soft caps to prevent over-protection of aging items) are uninvestigated.
- Does `lfu_counter` reset on stash demotion interact beneficially or harmfully with second-chance recovery when both are active simultaneously?
- Could timestamp-based promotion be rehabilitated by recording timestamps only on the first *hit* (not on insertion), eliminating creation-time contamination?
- Would frequency-based eviction, ghost sets, or probationary zones win on a different workload trace with heavier scan traffic or bursty cold-item arrivals — where the access distribution is not uniformly warm?
- Is the current optimal geometry (24/8) robust across capacity regimes, or does a smaller total cache force a different hot/stash ratio?

## Per-policy notes

- **ClassicLRU / position-based recency:** Dominant throughout all cycles on workload-a. MRU insertion is consistently optimal; any deviation toward LRU/Front positioning regresses. This policy class should be the baseline for all future variants.
- **LFU / frequency-based discrimination:** Structurally incompatible with workload-a. Classic LFU, aging-LFU (aggressive and threshold-gated), and raised frequency-guard thresholds all failed. Single-touch items carry genuine reuse signal and must not be treated as cold. Retire for workload-a; revisit only if a scan-heavy or frequency-skewed trace is introduced.
- **Ghost / victim-cache re-admission:** Noise-level across every position (MRU, Back-insert) and trigger variant tested. The stash eviction boundary is not where meaningful hit-rate is lost. Permanently retired.
- **Zero-access frequency guard (eviction selectivity):** The most impactful late-search gain. Protecting any item with at least one confirmed in-tier hit from eviction by zero-access newcomers is both necessary and sufficient — the zero-access cutoff is exactly right for this workload. Active in the current champion.
- **Second-chance stash overflow recovery:** Real, positive, additive with the frequency guard because it targets a different pipeline stage (stash overflow vs. normal-tier eviction ordering). Active in the current champion.
- **TimestampLRU:** Partially replicates topology and MRU-insertion gains already captured by prior winners; does not surpass them, and interacts destructively with frequency-guard eviction. Deprioritised unless a creation-time-clean variant can be specified.
