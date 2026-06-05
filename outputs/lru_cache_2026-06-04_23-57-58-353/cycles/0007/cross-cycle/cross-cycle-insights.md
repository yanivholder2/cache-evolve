# Cross-Cycle Insights

## Recurring wins

- **Two-segment SLRU is the load-bearing structural gain.** Every winning baseline since cycle 1 has been built on protected/probationary segmentation; no cohort has found a way to match it without it.
- **Applying the eviction signal at both segment boundaries pays off.** Cycle 4 confirmed that frequency-aware demotion in protected while using blind LRU in probationary left a gap; closing the asymmetry produced a clear gain. The pattern generalised to IRR in cycle 7.
- **Signal preservation across demotion is load-bearing.** Counter halving on demotion (cycles 3–6) and IRR-timestamp preservation across demotion (cycle 7) both showed that continuity of the warmth signal across the segment boundary is required for the signal to remain useful in probationary.
- **Sharper signals consistently dominate coarser ones.** IRR (last-two-accesses recency gap) beat accumulated frequency counts in cycle 7 by a wider margin than frequency counts beat plain LRU recency in cycle 1. The LIRS-style trace rewards temporal clustering, and policy quality tracks how precisely the signal captures that.
- **Neutral wins are real convergence signals.** Two or three consecutive cohorts where the best variant advances on neutrality (cycles 5–6) reliably indicated the policy had reached a local optimum before IRR broke the plateau.

## Recurring failures

- **Ghost admission across every window size tested.** Probationary-sized window (cycles 2–3): negligible lift, confirmed negligible in cycle 3 isolation. Half-total-capacity (cycle 6): regressed. Full-capacity (cycle 4): regressed harder. Reuse events at distances beyond probationary capacity on this workload are cold traffic, not deferred-warm traffic; window resizing cannot rescue the mechanism on this trace.
- **Eviction-site-only frequency mechanisms.** The cycle-1 frequency-window scan was bounded by its placement: acting only at eviction time, after warm entries were already candidates, is structurally weaker than acting at insertion (SLRU). The initial implementation also failed from a small fixed window and counter reset; both flaws are now understood.
- **Fixed-rate frequency decay.** Logical-clock halving at a fixed half-life (cycle 6) collapses the frequency distinction between demoted-protected entries and cold newcomers whenever the half-life is shorter than the workload's median inter-access gap. Any fixed constant is likely wrong without workload-specific tuning.
- **Mechanisms that suppress or freeze node position updates.** The cycle-3 second-chance bit that froze protected-segment node positions broke LRU ordering and hurt performance. Continuous move-to-tail on every access is load-bearing; any reprieve mechanism must preserve it.
- **Scan-fraction widening beyond the established thresholds.** Widening the protected scan from 1/4 to 1/3 (cycle 5) and various wider probationary scans produced negligible or no returns, indicating frequency variation near the LRU end is already captured by the narrower window on this trace.

## Open questions

- **Does IRR-based eviction transfer beyond the LIRS trace?** All gains were measured on a single workload. Multi-workload evaluation is overdue and is the clearest way to distinguish broadly strong policy from workload-specific alignment.
- **Is the 80/20 protected/probationary split still optimal under IRR?** The split has been fixed since cycle 1 and stress-tested only for frequency-based policies. Adaptive segment-ratio balancing (H01) has never been evaluated and is now more interesting on the IRR baseline.
- **Are the current scan-window fractions well-calibrated for IRR?** IRR is a sharper discriminator than frequency counts; narrower scan fractions might yield equivalent eviction quality with less overhead. This is unexplored on the new baseline.
- **Can a second-chance reprieve be salvaged without position-freeze?** The cycle-3 failure was caused by suppressing move-to-tail, not by the reprieve concept itself. Setting a referenced bit while still performing the position update was not tested.
- **Is there a fixed decay constant that preserves IRR distinctions on workload-a?** It is unclear whether the decay-collapsing problem is fundamental on this trace or specific to the cycle-6 half-life choice; it remains untested on the IRR baseline.

## Per-policy notes

- **Plain LRU**: Scan-pollution vulnerability confirmed real and consistently exploitable. Serves only as the initial reference baseline.
- **CLOCK (cycle 1, v002)**: Eliminates linked-list pointer maintenance; performed between LRU and SLRU. Whether its structural simplification yields throughput advantages under high concurrency has never been measured.
- **SLRU 80/20 (cycles 1–7 baseline)**: Durable foundation. Widening probationary to 30% hurt in cycle 2 — the working set is large enough that the smaller protected segment causes more churn than the extra scan-filtering headroom saves. Ratio has not been revisited since.
- **SLRU + ghost admission**: Marginal contribution confirmed negligible in cycle-3 isolation test. Dropped from the baseline after cycle 7 with no regression.
- **SLRU + frequency-aware demotion + counter halving (cycles 3–6)**: Full design closed in cycle 4 (symmetric eviction signal at both sites) and capped in cycle 5 (MAX_FREQ=8 guardrail, neutral). Three consecutive neutral-winner cohorts confirmed workload-a convergence. This policy line is exhausted on this trace.
- **SLRU + IRR timestamps (cycle 7, current baseline)**: Largest single-cycle hit-rate gain in the run. IRR preserved across demotion replaces counter halving naturally; cold newcomers arrive with IRR = MAX and lose the eviction contest without manual counter adjustment. Ghost set dropped. Scan fractions, segment ratio, and multi-workload behaviour all remain untested on this baseline.
