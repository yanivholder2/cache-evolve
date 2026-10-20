## Recurring wins

- **Two-segment SLRU with 80/20 protected/probationary split** holds as the load-bearing structure across all nine cycles; every winning design preserves it and every attempt to widen probationary beyond 20% regresses.
- **Frequency-aware eviction scanning the LRU quarter of protected and LRU half of probationary** reliably beats blind LRU eviction at both sites; narrowing the probationary scan consistently degrades quality because frequency/IRR variation is spread throughout the second quarter, not compressed near the LRU head.
- **Signal continuity across segment transitions** — counter halving on demotion (frequency baseline) or IRR timestamp preservation (IRR baseline) — always pays off; resetting any accumulated signal on demotion discards the very information that makes intra-segment ordering competitive.
- **Acting at insertion time (probationary triage) rather than only at eviction time** was the structural advantage of SLRU over CLOCK and frequency-window scan from cycle 1 onward; subsequent wins have consistently extended or refined the eviction-time signal rather than the insertion-time one.
- **Frequency counting as primary eviction signal** outperforms pure IRR in absolute hit-rate terms on workload-a; the IRR baseline produced real relative gains within its own search but never reached the frequency baseline's absolute ceiling.

## Recurring failures

- **Ghost window expansion beyond probationary capacity** — tested at half-total (cycle 6) and full-total (cycle 4) capacity; both regressed; reuse events at those distances are cold traffic on workload-a, not deferred warmth.
- **Secondary tie-breaker signals on top of a dominant primary signal** — IRR tie-breaking over frequency ties (cycle 9 v001) and visit-count tie-breaking over IRR ties (cycle 9 v002) both inert; equal-candidate collisions under the dominant signal are too rare to matter.
- **Fixed-rate logical-clock decay on frequency counters** (cycle 6) — any single decay constant shorter than the workload's inter-access gap collapses the frequency distinction between demoted and cold entries and eliminates the scan's protective value.
- **Second-chance reprieve with position-freeze** (cycle 3) — severing the continuous recency ordering in the protected segment hurts more than the reprieve saves; the cycle-3 failure was conclusive.
- **Widening probationary to 30%** — regressed in cycle 2 and again in cycle 8 (IRR baseline); the hot working set is dense enough that a 70% protected segment is genuinely insufficient.
- **Ghost admission combined with IRR-based eviction** (cycle 8) — non-complementary; IRR loses the long-term access history that frequency halving preserved, so ghost-set promotion shortcuts insert the wrong entries into protected.
- **Narrowing the probationary scan below half** (cycles 5, 8) — always regresses; confirmed load-bearing on both frequency and IRR baselines.

## Open questions

- **Does restoring ghost admission to the current frequency-primary baseline recover the cycle-9 regression and yield net gain?** Ghost was stripped from cycle-9 v001 and the regression was absorbed; the marginal value of ghost within a pure frequency design has not been cleanly isolated since cycle 3.
- **Is staleness-weighted frequency scoring (or staleness-weighted IRR as a secondary) additive with the frequency-primary signal?** Cycle-9 v003 showed a modest directional gain on the IRR baseline; the interaction with frequency-primary is untested.
- **Adaptive segment-ratio balancing (ARC-style feedback)** — H01 was blocked in cycle 7 and never rescheduled; with the baseline now stable it is the main unexplored structural axis.
- **Multi-workload transferability** — the entire policy search has run on workload-a only; whether gains generalize to non-LIRS traces has been flagged as overdue since cycle 5 and remains unaddressed.
- **IRR baseline with ghost admission** — the IRR baseline was never given ghost admission, which was a net-positive mechanism in the frequency design; that comparison was never made directly.

## Per-policy notes

- **Plain LRU**: scan-pollution failure confirmed cycle 1; one-hit entries displace warm entries during cold bursts; baseline for all comparisons.
- **CLOCK (visited-bit, single-segment)**: cycle 1 only; no linked-list overhead is a theoretical throughput advantage, but hit-rate is weaker than SLRU; not revisited.
- **SIEVE (visited-bit second-chance)**: cycle 7 only; meaningful gain over LRU but smaller than IRR on the same cycle; no further evaluation.
- **SLRU frequency baseline**: the dominant design thread (cycles 1–6, 9); MAX_FREQ=8 cap is a harmless guardrail; counter halving on demotion is load-bearing; scan fractions (1/4 protected, 1/2 probationary) are stable and confirmed from both directions.
- **Ghost admission on frequency baseline**: positive in cycle 2 (though contribution was initially over-attributed); confirmed net-positive in isolation only in cycle 3 context; stripped in cycle 9 without a clean re-isolation; restoration is the top open thread.
- **IRR-based eviction**: cycle 7 produced the largest single-cycle gain in the series but from a lower absolute floor; IRR timestamp preservation across demotion naturally replaces counter halving; probe fractions and segment ratio confirmed identical to frequency baseline; staleness weighting (cycle 9 v003) is a modest validated improvement direction within the IRR framework.
- **Signal-blending (frequency + IRR hybrid)**: cycle 9 confirmed that additive secondary signals are inert when the dominant signal resolves most eviction contests unambiguously; blending is only productive if both signals operate in regimes where the other is weak.
