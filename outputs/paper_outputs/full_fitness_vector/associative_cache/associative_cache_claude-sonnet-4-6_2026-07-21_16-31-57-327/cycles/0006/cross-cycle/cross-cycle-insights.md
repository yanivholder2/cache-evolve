## Recurring wins

- **32-byte aligned struct as structural base.** Padding the compact entry to 32 bytes eliminated cache-line straddling and halved p99 latency (Cycle 0002). Every subsequent winner has carried this layout forward; dropping it immediately inflates footprint and erases other gains.
- **3-swap bounded promotion loop with early-break.** A cap of three positional swaps per get, with an early exit on a failed comparison, lets Zipfian hot keys converge to the head in one iteration while giving mid-bucket items enough budget to reach the front zone. Single-swap was too conservative (Cycle 0003); five swaps caused regression by displacing recently-active items (Cycle 0006). Three is the confirmed optimum.
- **Strict tail eviction (no frequency signal at eviction time).** Pure positional LRU from the tail consistently outperforms frequency-overlaid eviction on this workload. Items near the tail are genuinely cold given the Zipfian distribution; every attempt to override positional order at eviction time has degraded hit rate.
- **Write-once binary "ever-accessed" flag for second-chance eviction.** A flag written only at first access — never on the hot get path — allows the eviction scan to skip warm items and eject true newcomers. Confirmed hit-rate lift in Cycle 0006 with no per-get write cost.

## Recurring failures

- **Per-get frequency counter writes.** Every variant that incremented an `lfu_counter` on the hot get path surfaced p99 regression (Cycles 0001, 0002, 0005, 0006). The write itself — not the conditional logic built on it — is the cache-line liability. Frequency data collected on every get is a consistently closed direction.
- **Two-zone (hot/cold) partitioning.** The hot/cold boundary shifts with every insertion, pushing recently-promoted items back into the cold zone. Tested thoroughly in Cycle 0003 and definitively closed; no reformulation at these bucket sizes has been proposed that would change the structural problem.
- **Swap-to-back promotion (swap instead of remove+push).** Degraded both hit rate and latency simultaneously at small associativity; the linear shift it was meant to avoid is not the bottleneck. Closed in Cycle 0002.
- **Reducing bucket count.** Halving buckets raises collision density faster than depth can absorb it. Regressed hit rate in Cycles 0001 and 0006 with no compensating gain. Closed.
- **Promotion budget beyond three swaps.** Five swaps (Cycle 0006) produced a slight hit-rate regression by displacing items that cycle near the head on Zipfian workloads. The ceiling is mapped; further budget expansion is not worth retesting.

## Open questions

- **Can second-chance eviction preserve its hit-rate lift when paired with zero or single-swap promotion?** Cycle 0006 identified that the flag and the 3-swap loop are mechanistically independent. The next experiment should decouple them: try (a) flag + one swap, (b) flag + no promotion. If either holds hit rate near the 3-swap+flag baseline, it becomes the new confirmed base with lower latency cost.
- **Is the per-get write volume the precise threshold, or does write *frequency* (e.g., sampled writes) allow frequency data on the get path at acceptable cost?** Cycle 0005 suggested a general threshold exists; it has not been quantified. Epoch-based or sampled counter updates might thread the needle.
- **Does the measurement floor still mask latency improvements on write-light get paths?** Cycle 0004 flagged 100 ns timer resolution as a ceiling. Cycle 0005's p99 doubling on write-heavy paths shows differentiation is observable under load — but whether small improvements on write-light paths (e.g., the 3-swap base vs. a zero-swap variant) are visible remains unresolved.

## Per-policy notes

- **LRU (positional, no frequency).** The 3-swap unconditional promotion loop with the 32-byte aligned struct is the current best-confirmed base. Get path carries no writes beyond the swap(s) themselves; eviction is strict tail removal. Two cycles of Winners emerged from this configuration.
- **LFU-semantics (per-get counter).** Hit rate is consistently demonstrated on scan-heavy and Zipfian traces, but per-get counter writes reliably surface latency regression across Cycles 0001, 0002, 0005. The information has value; the write location does not. Any future LFU integration must defer writes off the hot get path entirely.
- **Second-chance / generational.** Write-once flag at first access is the promising variant (Cycle 0006 Winner). Pairing with a lighter promotion strategy is the immediate next step. Two-zone hard partitioning is a closed cousin of this idea and should not be conflated with it.
- **Two-tail / frequency-at-eviction hybrids.** Tested across Cycles 0004, 0005, 0006 with consistent hit-rate degradation. Overriding strict positional order at eviction time consistently ejects warm items prematurely on Zipfian workloads. This hybrid family is closed unless a fundamentally different eviction-signal encoding is proposed.
