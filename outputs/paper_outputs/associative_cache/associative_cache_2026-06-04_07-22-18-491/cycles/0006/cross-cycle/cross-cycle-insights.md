## Recurring wins

- **Composing complementary mechanisms**: every winning composition targeted non-overlapping failure modes. LFU eviction selects a better victim; the FIFO stash recovers displaced warm items; probe admission blocks cold items before they compete for primary slots. Each layer added value only because the others handled what it could not.
- **Structurally minimal changes that activate latent capacity**: the first win (LFU counter) piggybacked on a field already carried at zero cost. Later wins built on the same principle — changes that reuse existing structure consistently outperform changes that add new state.
- **Composition hypotheses validated by the ledger itself**: both Cycle 2 and Cycle 6 confirmed compositions explicitly recommended by the prior cycle. Following the ledger's own open questions has a high hit rate.
- **Probationary admission as the strongest single lever**: moving the filter to the admission boundary (before items enter primary) outperformed all eviction-side refinements tested across five preceding cycles, and did so from a simpler baseline.
- **Wins cluster at structural boundaries**: the highest-gain changes operated at transitions — primary-to-stash (promotion credit), outside-to-primary (probe admission) — rather than inside a single subsystem.

## Recurring failures

- **Eviction-scoring refinements on a stable Zipfian hot set**: counter caps and frequency-times-recency product scores were never triggered by workload-a, whose hot-set membership is stable. Sound logic measured as neutral because the failure modes it targets do not occur on this trace.
- **Promotion credit above +2**: flat +3 and frequency-proportional scaling both overshoot the one-burst-survival threshold. Counter-derived heuristics at the promotion boundary are unreliable because past LFU values reflect burst-displacement timing, not future reuse probability.
- **Stash intake filters**: rejecting zero-counter evictees removes recently inserted items displaced before their confirming access, not cold waste. Stash value depends on holding all recent evictions, not a quality-filtered subset.
- **Larger stash caps**: doubling the quarter-capacity default let cold items linger and earn spurious promotions, displacing medium-frequency primary residents. The FIFO cap is a self-cleaning mechanism, not a constraint.
- **LFU ordering inside the stash**: every stash resident already arrived as the weakest LFU item in primary; the frequency gradient within the stash is too compressed to discriminate. FIFO's age-based ordering is better matched to the stash's role.
- **Ghost / extended eviction memory**: giving re-inserted items a frequency head start based on ghost-tier history inverts the LFU signal on this workload and costs hit rate.

## Open questions

- **`metadata-size` evaluator gap (six cycles unresolved)**: confirmed in Cycle 5 as a harness infrastructure problem — the `evolve_metadata_size` binary is not wired into the evaluation pipeline. Probe, stash, and LFU counter fields add per-bucket memory overhead that remains entirely invisible to the fitness function. Any memory-efficiency hypothesis is unverifiable until this is fixed.
- **Generalization beyond stable Zipfian workloads**: counter-cap, product-score, and ghost-tier hypotheses may be sound but untriggered. A trace with hot-set drift or bursty insertion pressure could surface gains that workload-a systematically suppresses.
- **Probe zone sizing above half-capacity**: half-capacity is at or near the minimum viable window for this workload's inter-access gap. Whether the gain is monotone above that point or peaks near half-capacity has not been tested.
- **Structural axes outside the probe-stash-LFU triangle**: six cycles have converged on one mechanism family. Key-space partitioning, time-windowed frequency, and workload-shift detection are unexplored directions that may matter more on non-stationary workloads.
- **Dynamic promotion credit**: a multiplier that scales with how many times an item has cycled through the stash was raised but never tested; it may outperform the fixed +2 under sustained eviction pressure.

## Per-policy notes

- **ClassicLRU (baseline)**: cannot distinguish hot from cold traffic under cold-burst conditions; all subsequent gains trace to mechanisms that add discrimination the baseline lacks.
- **LFU-weighted eviction**: neutral in isolation once probe admission is in place — the frequency distribution within primary is too flat after probe filtering. Only valuable in combination with the stash, where it selects a better victim for the stash to hold. The `lfu_counter` field was already present at zero cost; the win came from activating it, not adding it.
- **FIFO stash at quarter-capacity**: correct internal ordering — LFU within the stash is redundant and adds no discrimination. The cap is calibrated; larger caps accumulate noise. The stash's role is bounded recovery of warm items at the primary boundary, not a secondary cache in its own right.
- **Promotion credit +2**: confirmed local optimum across multiple probes. +1 loses promoted items to immediate re-eviction on the next cold burst; +3 and higher overshoot. The optimal value is workload-specific and tied to the typical burst length.
- **Probe-zone admission at half-capacity**: strongest mechanism found. Half-capacity is the minimum window that allows a confirming re-access to arrive before the probe FIFO ejects the item. Quarter-capacity regressed; above half has not yet been tested.
- **Full composition (probe + stash + LFU eviction + promotion credit)**: the mechanism set as of Cycle 6 baseline. Each layer addresses a different failure mode; removing any single element degrades the outcome. The design appears locally complete on this workload.
