## Recurring wins

- **Hot-tier topology expansion paid off across many cycles.** Shifting capacity from stash to normal buckets was the highest-leverage lever from C1 through C8, with each increment producing a real gain until the ceiling at a roughly seven-eighths normal fraction was confirmed in C9. Returns shrank monotonically with each step; the gain curve was concave throughout.
- **MRU insertion is universally correct on workload-a.** Every variant that inserted at the MRU end of its target tier outperformed or held steady; every attempt to withhold MRU status (probationary positioning, textbook demotion cascades) regressed. Recency position is a stronger future-access signal than any guard or counter for this workload.
- **Zero-access eviction guard (binary frequency threshold) is a clean win.** Protecting any item with at least one in-tier hit from being displaced by zero-access newcomers produced a clear improvement (C6) and compounded additively with second-chance stash recovery (C7). The binary threshold is the correct abstraction — the counter should never be promoted to a ranking signal.
- **Second-chance stash recovery at Front/LRU is additive.** Unconditionally re-routing stash-evicted items to the LRU end of their target bucket stacks cleanly with the frequency guard because it targets a distinct pipeline stage (stash overflow vs. probing-bucket eviction). Front insertion correctly treats re-admitted items as probationers; Back/MRU insertion regressed (C8).
- **Pre-insertion counter credit on confirmed-access paths closed a real protection gap.** Crediting `lfu_counter` before re-insertion at the stash-hit exit eliminated a window where a just-accessed item re-entered the target tier looking cold. Genuine but modest gain (C9).

## Recurring failures

- **`lfu_counter` as a ranking signal consistently hurts.** Frequency-ranked eviction, frequency-gated promotion, and frequency tiebreakers all regressed or were neutral (C1, C3, C10). The counter cannot distinguish newly-inserted items from genuinely cold ones; once the binary zero-vs-any threshold is crossed, recency order dominates.
- **Aging-LFU in any form is contraindicated on workload-a.** Coarse symmetric decay evicted mildly-warm items with residual heat (C2); threshold-gated softer decay replaced that failure mode with a slow-ramp blindspot that still underperformed ClassicLRU (C3). Workload-a's distribution is recency-dominant and punishes frequency discrimination regardless of aggressiveness.
- **Ghost/victim-cache re-admission produced noise across all configurations.** Tested in multiple cycles with different insertion positions and ghost-set sizes; the stash eviction boundary is not a meaningful hit-rate loss site on this workload. Retired after C5.
- **Stash bypass or weakening always degraded quality.** Any variant that shortened stash traversal — front-door bypass, LFU-gated promotion delay — allowed cold or insufficiently-proven items to displace hot residents (C4). The stash is a load-bearing admission filter for this workload.
- **Timestamp-based promotion is contaminated by insertion time.** Selecting promotion candidates by maximum `last_accessed` allows items whose creation timestamp postdates a hot item's last-access time to win promotion undeservedly. The resulting cold-item leak cancels or exceeds any eviction-selectivity gain (C6 v003, C7 v002).
- **Geometry expansion past the seven-eighths ceiling regressed.** Reducing stash to fewer than four slots leaves insufficient headroom for cold-arrival bursts; second-chance recovery cannot compensate for the lost stash capacity (C9).

## Open questions

- **Soft-capping `lfu_counter` is the one counter-management axis never tested.** If long-lived hot items accumulate unboundedly high counts, newly warm items that should displace them cannot. This has been carried forward since C7 and remains unprobed; it could matter or be irrelevant at current geometry.
- **Probing-bucket demotion handling has not been examined.** With topology, insertion position, eviction selectivity, and counter accounting all explored, structural asymmetry in how items are ordered after demotion from the normal tier into probing is the remaining uninvestigated pipeline stage.
- **Would failed mechanisms compete on a different workload?** Ghost sets, frequency-ranked eviction, and admission gating all have plausible theory behind them. A scan-heavy or bursty cold-arrival trace might expose loss sites where these mechanisms pay off — the repeated finding that the stash is a quality gate for workload-a specifically suggests the workload is unusually stable.
- **Is there a proportional or access-count-weighted re-entry position that extracts more from the insertion-position lever?** Full Back-insert is confirmed optimal, but a graduated placement based on accumulated hit evidence has not been tested.

## Per-policy notes

- **ClassicLRU / MRU-insertion baseline:** Dominant throughout; position encodes a stronger reuse signal than any counter on this Zipfian, recency-dominant workload. Any variant that degrades insertion position regreses.
- **ClassicLRUFreqProtect (zero-access guard + second-chance + counter credit):** Current champion after C9. The binary zero-count threshold is the right abstraction. Do not extend the counter to ranking; do not change second-chance insertion from Front to Back.
- **LFU variants (ranking):** Structurally incompatible with workload-a; recency order is more predictive than any frequency counter formulation tested. Should only be revisited with evidence from a different workload trace.
- **Aging-LFU (coarse or threshold-gated decay):** Actively harmful regardless of decay rate; retired after C3.
- **TimestampLRU:** Partially replicates gains already captured by topology and MRU insertion without surpassing them. Incompatible with frequency-guard eviction due to creation-time promotion contamination. Deprioritised.
- **Ghost/victim cache:** Noise-level across every configuration and insertion position tested. Permanently retired.
