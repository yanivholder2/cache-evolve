# Cross-Cycle Insights

## Recurring wins

- **Small-queue capacity transfer to main.** Shrinking the small queue from the default 25% down through 10%, ~7%, ~5% consistently raised hit rate across cycles 1–10. Each step transferred higher-quality residency to main. The benefit is monotone but flattening; the empirical floor has not yet been breached above ~3.6%.
- **Ghost queue precision at main_size/2 cap.** A ghost window sized to main_size/2 outperformed both larger and smaller caps. The workload's re-use distances fit exactly inside this window; both directions have been tested and confirmed harmful.
- **Unconditional move-to-front on main-queue hits.** MTF prevents positional drift toward the eviction tail. Its value is incremental on a Zipfian workload (most hot items are rescued reactively anyway), but it is a real, consistent positive that stacks additively with queue-sizing gains.
- **MAX_FREQ ceiling above the baseline.** Raising from 3 → 4 produced a genuine gain; 4 → 5 was marginal but non-negative. The hot set is temporally sticky enough that a minority of items legitimately exercises extra retention passes.
- **Orthogonal mechanisms compound additively.** The ~5% small queue and unconditional MTF target distinct failure modes (cold-admission pressure vs. positional drift) and their combination outperforms either alone, confirmed in cycle 10.

## Recurring failures

- **Relaxing the promotion threshold (freq > 0 or lower).** Tried in cycles 1, 3, and 4 — solo and in combination. The narrow small queue already filters cold traffic; widening admission into main introduces churn without a corresponding benefit in re-used traffic. This axis is closed.
- **Ghost queue enlargement or redirection.** Extending the ghost window (cycle 1) and redirecting ghost population to track main-queue evictions instead of cold small-queue ejections (cycle 10) both hurt. More coverage does not mean better signal; it means more noise from items whose re-use tenure has already ended.
- **Ghost re-admission frequency credits.** Seeding ghost-re-admitted items at freq=1 (cycle 4) displaced incumbents that earned their slots, producing a net loss. The ghost signal justifies re-admission but not a residency head-start.
- **Second-chance threshold tightening after eager promotion.** Raising the evict_main gate in a configuration where eager promotion already deposits items at freq==2 turns the filter into a hot-item ejector rather than a stale-squatter filter. Cross-feature interactions can invalidate axes that were profitable in earlier structural states.
- **Fine-grained positional manipulation within main.** Midpoint insertion, saturation-triggered head-repositioning, and CLOCK-style in-place scan (cycles 5–6) all found that the hot set rarely satisfies the conditions required to benefit, and the baseline's reactive recency recycling already handles the common case. Positional tuning is not a lever.
- **Adaptive threshold control on a phase-stable workload.** The phase-sensitive controller (cycle 7) fired on noise rather than real phase shifts because workload-a is Zipfian and temporally stable. Adaptive mechanisms need workload diversity to function.
- **Extended freq==1 recycle window in evict_main.** Granting cold-tail items an extra pass (cycle 9) delays fresher admissions without improving their hit probability — the freq>1 threshold is already well-matched to the post-eager-promotion frequency distribution.

## Open questions

- **True floor for small-queue sizing.** The monotone trend holds through ~3.6%, but the floor has not been breached. Does the benefit cliff suddenly, or taper gradually toward some minimum where the cold filter becomes too coarse?
- **MTF + further queue shrinkage interaction.** Combining MTF with ~3.6% (rather than ~5%) has not been directly measured. Given that both mechanisms have been confirmed as additive, does the combination yield a further step, or does MTF's value diminish once the queue is already very tight?
- **Adaptive mechanisms on phase-shifting workloads.** The phase-sensitive threshold controller failed on workload-a but may be correctly designed for a workload with genuine scan bursts or hot-set transitions. Its value is an open question for other trace types.
- **Algorithm-level ceiling.** Seven cycles of structural probing found no gap beyond the S3-FIFO design on workload-a. Whether the residual performance gap is explained by workload entropy or by a property that a different algorithm (TinyLFU, SIEVE) exploits has not been benchmarked.

## Per-policy notes

- **S3-FIFO / workload-a:** The dominant axis across all cycles is small-queue capacity allocation — it is both the first lever to saturate and the one with the most headroom. Ghost sizing is a secondary but independently confirmed lever; the main_size/2 boundary is empirically determined from both sides. MTF is a third, orthogonal lever that stacks cleanly on top of the other two.
- **Eviction ordering is load-bearing, not cosmetic.** The cycle-5 CLOCK regression demonstrated that the implicit recency sort embedded in move-to-front is the primary mechanism keeping recently re-accessed items away from the eviction frontier. Removing it produced a large regression; refinements to it produced near-zero effects. It should be treated as structural, not tunable.
- **Eager promotion changes the main-queue entry distribution.** Once eager promotion was introduced, the frequency distribution of newly-arriving main-queue items shifted upward, invalidating any threshold logic calibrated against a pre-promotion baseline. Axes that looked exhausted or closed can reopen — or close more firmly — after structural changes of this kind.
- **The configuration as of cycle 10 (~5% small queue + MTF + MAX_FREQ=5 + main_size/2 ghost cap) appears near the local optimum for this workload.** All plausible static-parameter and structural axes within S3-FIFO have been mapped; further meaningful gains likely require either a lower small-queue floor or a different algorithm.
