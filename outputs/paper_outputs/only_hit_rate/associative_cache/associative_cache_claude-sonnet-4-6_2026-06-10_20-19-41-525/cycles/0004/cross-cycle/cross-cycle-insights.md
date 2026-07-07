# Cross-Cycle Insights

## Recurring wins

- **Bucket geometry widening improves hit-rate on locality-heavy workloads.** Expanding from narrow to wider per-bucket slot counts consistently reduced premature eviction of live items across cycles 0001–0002. The gain is front-loaded: the first widening step (factor ~4) captured most of the benefit; further pushing saturates quickly.
- **Cold-first eviction is the dominant eviction-quality lever.** Preferring `counter == 0` candidates reliably shields warm items from scan displacement. Introduced in cycle 0002, confirmed effective in cycles 0003–0004, and robust against reasonable threshold variants.
- **Targeted, infrequent counter decay self-corrects warm-lock without disrupting the normal path.** Decrementing counters only when no cold candidate exists is sufficient to restore cold-first selection in steady state. The mechanism is low-overhead precisely because it fires rarely.
- **LFU warm-fallback (minimum-counter selection) adds independent value on narrow geometry.** When cold pool exhaustion is frequent, replacing the LRU-position fallback with minimum-counter selection produces a substantial hit-rate gain orthogonal to geometry or insertion-site choices.

## Recurring failures

- **O(1) swap-to-back promotion for ClassicLRU.** Corrupts the recency gradient across intermediate slots; the hit-rate regression is large and consistent. The O(n) cost of correct LRU ordering is worth paying — revisiting this angle has not paid off.
- **Tightening the cold threshold beyond `counter == 0`.** Raising the warmth boundary to `counter ≤ 1` caused premature eviction of genuine hot-set members on workload-a. A single access is a reliable reuse signal on this trace; threshold tightening regresses and should be considered closed.
- **Pure geometry widening beyond ~32 slots per bucket.** The marginal hit-rate return diminishes to near-zero above factor ~4 (≈32–64 slots). Cycles 0002 and 0004 both confirm this axis is saturated for mid-point insertion paths.

## Open questions

- **Can counter decay (cycle 0004 winner) and LFU warm-fallback be stacked productively?** The two mechanisms are orthogonal — one restores cold supply, the other improves warm eviction quality — and have only been tested independently. Stacking them is the highest-priority unprobed combination.
- **What is the per-op latency cost of mid-point insertion (`vec.insert` O(n)) and cold-first scan relative to their hit-rate gain?** No latency evaluator has run on any variant; this remains an open measurement gap across all cycles.
- **Do gains from cold-first eviction, mid-point insertion, and counter decay generalise beyond workload-a?** All wins are measured on a single LIRS trace with a specific locality pattern. Cross-workload validation has not been attempted.
- **Does the mid-point-insertion-as-geometry-substitute trade-off hold under different access patterns?** Cycle 0003 showed that mid-point insertion can compensate for narrow buckets on workload-a; whether this substitution generalises is unknown.
- **Does counter decay remain stable over very long traces?** Periodic bulk decrement could in theory oscillate counter values and destabilise the cold/warm boundary on sustained workloads.

## Per-policy notes

- **ClassicLRU (baseline):** Narrow-bucket geometry is a confirmed bottleneck on scan-heavy workloads. Strict O(n) LRU ordering is load-bearing for eviction quality; O(1) approximations that discard the recency gradient consistently regress.
- **Wide-bucket ClassicLRU (geometry variant):** Effective but front-loaded. The ~4× bucket factor from cycle 0001 captures most of the geometric benefit; further widening yields diminishing returns. This variant is now the de-facto baseline for further eviction-strategy experiments.
- **Cold-first eviction (v002 lineage):** The strongest single improvement found so far. Depends on wide buckets as a cold-pool enabler, though mid-point insertion can partially substitute on narrow geometry. The `counter == 0` threshold is settled for workload-a.
- **Mid-point insertion (v003 lineage):** A secondary lever that overlaps significantly with cold-first eviction when both are active. Useful as a standalone mechanism on narrow geometry; provides only marginal additional gain when stacked on top of cold-first on wide geometry. Carries an unmeasured O(n) insertion-latency cost.
- **Counter-decay + cold-first (cycle 0004 winner):** Addresses the steady-state warm-lock failure mode. Self-correcting, low-overhead, and orthogonal to the insertion-site axis. Best current configuration; untested combined with LFU warm-fallback.
- **LFU warm-fallback (cycle 0004 v002):** Strongest improvement on narrow-geometry paths where cold exhaustion is routine. Independently developed from the counter-decay branch; the two have not yet been combined.
