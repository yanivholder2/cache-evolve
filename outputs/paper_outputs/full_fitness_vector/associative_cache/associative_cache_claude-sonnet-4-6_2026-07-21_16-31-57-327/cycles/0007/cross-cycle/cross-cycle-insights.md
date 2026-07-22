## Recurring wins

- **32-byte aligned struct is a required structural invariant.** Confirmed in cycle 0002 as a precise fix for p99 cache-line straddling; every variant that drops it reintroduces footprint bloat and put-path latency. Must be carried forward explicitly in every variant.
- **3-swap bounded promotion loop is the confirmed optimal promotion budget.** Early-break on a failed swap keeps the Zipfian hot-key path at one iteration. Cycle 0005 confirmed the win cleanly; cycle 0006 confirmed five swaps regress. The useful promotion-budget exploration range is exhausted.
- **Position-0 early-return guard reliably cuts tail latency.** Skipping the promotion loop for already-head items halved p99 in cycle 0003 without harming throughput. The guard is a safe, composable addition to any positional scheme.
- **Second-chance eviction with a write-once binary flag is the strongest hit-rate lever found.** First-touch marking lets the eviction scan skip warm items and target genuinely cold newcomers. It avoids hot-path write cost and won in cycle 0006 when paired with the 3-swap loop.
- **Strict positional tail eviction (no frequency signal at eviction time) consistently outperforms frequency-augmented alternatives** on the Zipfian/LIRS workloads used throughout. The eviction-quality gain comes from promotion ordering, not from counter-based selection at the eviction site.

## Recurring failures

- **Per-get frequency counter writes reliably introduce tail-latency pressure.** Tested in cycles 0001, 0005, and 0006. The write itself — not the conditional logic — is the cache-line liability. No frequency-counter approach applied on the hot get path has survived the latency constraint.
- **Two-tail / frequency-gated eviction selection consistently degrades hit rate.** Cycle 0005 (v002) and cycle 0006 both showed that comparing frequency at the eviction boundary ejects recently-promoted items before genuine LRU victims. This avenue is closed.
- **Two-zone (hot/cold) partitioning is definitively closed.** Boundary thrashing degrades hit rate faster than the hot-zone bypass saves work. Tested in cycle 0003 and not worth revisiting.
- **Swap-to-back / single-swap promotion is definitively closed.** Ordering approximation hurts hit quality on Zipfian; the linear shift it replaced was not the latency bottleneck at these bucket sizes. Closed in cycle 0002.
- **Bucket count reduction consistently hurts.** Fewer buckets raise collision density beyond what increased depth can absorb. Replicates across cycles 0001 and 0006; no reformulation has helped.
- **Decoupling second-chance scan from 3-swap promotion fails.** Exhaustively tested in cycle 0007 by reducing to one swap and zero swaps. Hot items age into the scan window alongside cold items when promotion is curtailed, stripping the scan of its discrimination power. The two mechanisms are coupled, not independent.

## Open questions

- **What caused the put p99 regression in cycle 0006 v003?** Swap-elimination experiments in cycle 0007 falsified promotion write-pressure as the cause. The regression source is undiagnosed and must be identified before the cycle 0006 v003 configuration can be declared a clean winner.
- **Is there a write-free mechanism to maintain hot/cold separation?** Positional promotion (3-swap) is currently the only confirmed way to keep hot items out of the eviction scan window. Whether a write-free alternative exists is unresolved.
- **Can frequency information reach the eviction path without per-get writes?** Epoch-based or put-time counters that update only on eviction or insertion — not on every get — remain untested. Cycle 0005 identified this as the natural next frequency experiment.
- **Does the 100 ns timer floor mask real latency differences below a meaningful threshold?** Cycle 0004 first surfaced this; cycle 0007 confirmed that eliminating all promotion swaps bought no measurable improvement. Distinguishing genuine parity from measurement noise is now a prerequisite for any further latency-targeted hypothesis.

## Per-policy notes

- **Positional LRU with bounded promotion**: The confirmed base configuration is: 32-byte aligned struct + position-0 guard + 3-swap unconditional loop + strict tail eviction. This combination won in cycle 0005 and serves as the stable foundation. Promotion budgets above three swaps regress; below three swaps sacrifice mid-bucket convergence.
- **LFU / frequency-ordered**: Frequency ordering's advantage is eviction-candidate quality, not get-path speed. Every attempt to maintain per-get counters has introduced latency risk. The write-once binary "ever-accessed" flag is the only frequency encoding that avoids hot-path write cost and produces a measurable hit-rate lift (cycle 0006 v002 winner).
- **Second-chance scan**: Mechanically coupled to the 3-swap promotion loop — the scan only discriminates hot from cold when promotion keeps genuinely hot items out of the scan window. Scan window size (8-slot) and flag placement (first-touch vs. position-0 arrival) are both confirmed as neutral on this workload.
- **Zone-based / multi-tail policies**: Two-zone partitioning and two-tail frequency comparison have both been tested across multiple cohorts and uniformly failed. These policy families should not be revisited without a fundamentally different structural motivation.
