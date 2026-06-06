## Recurring wins

- **Two-segment SLRU** (protected + probationary) has been the load-bearing structure since cycle 1; every winning design builds on it.
- **Frequency as the primary eviction signal** outperforms pure-IRR in absolute hit-rate terms on this workload; the frequency baseline's ceiling is higher than the IRR baseline's ceiling.
- **Counter halving on demotion** — preserving frequency history across segment boundaries rather than resetting — has been essential since cycle 3; reset counters make the signal evaporate.
- **Scanning the LRU half of probationary** (not the quarter) is load-bearing: IRR and frequency variation are distributed throughout probationary, not compressed near the LRU head.
- **Ghost admission** is a confirmed net-positive shortcut within frequency-primary designs; it acts at insertion time and is structurally orthogonal to frequency-based victim selection.
- **Staleness as a tie-breaker** over equal-frequency candidates has been validated on two separate baselines (IRR-primary in cycle 9, frequency-primary in cycle 10); it is defined from the first access and handles the single-access entry class that IRR cannot rank.
- The **80/20 protected/probationary split** has survived stress tests from both directions (cycles 2 and 8); it reflects the density of the active working set on this workload.

## Recurring failures

- **Ghost admission combined with IRR eviction**: ghost-admitted entries re-enter protected without a second timestamp, making their IRR score the maximum sentinel — indistinguishable from the coldest newcomer — and the two mechanisms undermine each other.
- **Widening probationary beyond 20%**: triggers excess demotion churn; the hot working set is dense enough that a protected segment below 80% is genuinely too small (cycles 2 and 8 confirmed independently).
- **Expanding the ghost window beyond probationary capacity**: reuse events at greater reuse distances are cold traffic on this trace, not deferred-warm arrivals; full-capacity ghost (cycle 4) and half-capacity ghost (cycle 6) both regressed.
- **Fixed-rate frequency or IRR decay**: any single decay constant collapses frequency distinction when its half-life does not match the trace's inter-access distribution; the failure is a parameter-matching problem but no constant has been found to generalize.
- **Narrowing the probationary scan to a quarter**: frequency and IRR values among demoted entries span the full second quartile; cutting the scan there loses meaningful selection pressure.
- **IRR as a tie-breaker over equal-frequency candidates**: equal-candidate collisions under the dominant signal are rare enough on this trace that secondary tie-breakers activate negligibly (cycles 9 and 10).
- **Second-chance reprieve bits that freeze node position**: severing continuous recency updates degrades the protected segment's ordering and offsets any reprieve benefit (cycle 3).
- **IRR as the sole eviction signal replacing frequency entirely**: two cycles of IRR-baseline optimization peaked below the frequency baseline; the IRR signal conflates historical and current warmth disadvantageously on this workload.

## Open questions

- **Additive combination of frequency + staleness + ghost admission**: cycles 9 and 10 validated each of the latter two independently on the frequency-primary baseline but never combined them; this is the highest-priority untested hypothesis.
- **Multi-workload transferability**: all gains have been validated on a single LIRS-generated trace; it is unknown whether the policy is broadly strong or specifically matched to this trace's reuse-distance structure.
- **Adaptive segment-ratio balancing** (ARC-style feedback): proposed repeatedly but never implemented; the fixed 80/20 split is robust but may not be optimal across varying workload intensities.
- **Staleness + ghost interaction**: ghost-admitted entries have a finite and growing staleness from their prior residency; it is unclear whether this makes staleness tie-breaking more or less effective in the ghost-admission population.
- **Whether any IRR-compatible ghost mechanism exists**: special-casing ghost entries at eviction time (e.g., giving them a synthetic IRR based on ghost-residency gap) might rescue the combination, but it has not been attempted.

## Per-policy notes

- **Plain LRU (baseline)**: scan-pollution vulnerable; any segmentation or reuse-aware mechanism improves on it; useful only as a floor.
- **CLOCK and frequency-window eviction (cycle 1)**: eviction-time-only signals; weaker than insertion-time segmentation on scan-heavy workloads; not revisited after cycle 1.
- **SIEVE visited-bit (cycle 7)**: suboptimal relative to IRR on this trace; not revisited.
- **SLRU + frequency (cycles 1–6, 9–10)**: the dominant policy line; counter halving, LRU-quarter scan in protected, LRU-half scan in probationary, and ghost admission all proven additive within this family; staleness tie-breaker validated as a further improvement; currently the best-performing design.
- **SLRU + IRR (cycles 7–8)**: strong relative gains from its own starting point but absolute ceiling below the frequency baseline; ghost admission is incompatible due to missing second-timestamp problem; scan fraction and segment-ratio conclusions mirror the frequency baseline.
- **Ghost admission**: net-positive within frequency-primary designs (cycles 2 and 10), neutral to harmful in IRR designs (cycles 3–6 on frequency, cycles 8–10 on IRR); the mechanism is design-context-dependent, not universally positive or negative.
