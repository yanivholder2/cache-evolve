## Recurring wins

- **Eliminating serial hash dependencies unlocks ILP gains.** Every hash-reduction winner (c0003 single-hash bit-slice, c0004 two-hash bit-slice, c0005 bijective remix) succeeded by removing inter-iteration data dependencies rather than simply reducing hash count. Variants that introduced serial chains regressed even when they issued fewer total hashes.
- **Bijective / cheap remixing is a viable substitute for full second hash invocations.** A three-instruction multiply-xorshift step provides sufficient row independence for this workload's accuracy requirements, making full SipHash calls wasteful on the put path.
- **Amortised sampling preserves latency while retaining a real signal.** A 1-in-8 eviction-probe rate yielded a stable, repeatable hit-rate improvement with no per-op-time cost. The signal survived sampling, confirming the frequency-comparison benefit is robust, not marginal.
- **Idiomatic Rust abstractions consistently outperform hand-rolled equivalents.** Option<NonZeroU32> match arms, standard hash APIs, and compiler-friendly discriminant tests beat explicit sentinel arithmetic and custom serial chains across multiple cycles.
- **Frequency accumulation is a reliable admission signal on stable working sets.** The LIRS trace's long-lived hot set means accumulated sketch counts are highly predictive; mechanisms that preserve them (no decay, no aggressive resets) have a structural advantage throughout.

## Recurring failures

- **Sketch decay at any tested interval degrades both hit rate and latency.** Two cadences — aggressive (width/2, c0001) and moderate (capacity×10, c0003) — both hurt: blanket half-decay penalises hot and cold keys equally, and the periodic full-table sweep pressures L2/L3. This axis is definitively closed.
- **Structural sketch simplification consistently backfires.** Flat single-allocation layout introduced cache-set conflicts (c0002); halving depth degraded collision suppression (c0002). The baseline's four-row multi-allocation design is load-bearing for this workload's capacity and Zipfian access pattern.
- **Node memory layout compaction yields no fitness win.** Compact 24-byte nodes (c0001–c0002) improved memory density but doubled hot-path latency; NonZeroU32 encoding (c0002) recovered parity but not a winner margin. The node-layout axis is exhausted.
- **Eager stricter admission gates disrupt cache residency in ways that dominate execution savings.** Applied on every eviction (c0004 v003, c0005 v003), a harder-to-satisfy threshold alters which nodes occupy cache lines enough to double p50 — an effect that amortised eviction sampling cannot cancel, because the two mechanisms target independent failure modes.
- **Tighter best-of-2 probe amortisation (1-in-4) breaks latency neutrality.** 1-in-8 is the upper bound; 1-in-4 regressed both p50 and p99 visibly (c0005). Further tightening requires reusing already-computed estimates rather than issuing new probes.

## Open questions

- **Softer admission gate on the amortised-probe path only.** Rather than applying a stricter threshold on every eviction, firing it only during the 1-in-8 sampled probe could limit residency churn while still capturing some hit-rate benefit. Untested.
- **Window and protected/probationary segment sizing.** No cycle has touched window ratio or the probationary-to-protected promotion threshold. These structural policy parameters are the largest remaining unexplored axis.
- **Reusing pre-computed frequency estimates to push amortisation finer.** The 1-in-8 rate is bounded by fresh probe cost; if eviction decisions could reuse a frequency value already computed during the same put operation, the amortisation ceiling might rise without new latency cost.
- **Two-phase hash with cheaper independence.** Can a bijective re-mix of the bijective-remix output (two remix steps from one SipHash) provide four sufficiently independent row positions, eliminating SipHash entirely from the sketch path?

## Per-policy notes

- **CountMinSketch (hash computation axis):** The productive trajectory runs single-hash bit-slice (c0003 winner) → two-hash bit-slice (c0004 winner, p99 cost) → bijective remix (c0005 winner, p99 recovered). Each step increased row independence or reduced serial dependency. Further hash-work reduction should focus on remix chaining rather than additional SipHash calls.
- **Eviction candidate selection:** Best-of-2 frequency comparison has a genuine hit-rate signal (first seen c0003 v003), but only amortised 1-in-8 keeps it latency-neutral (c0004 winner). The 1-in-4 rate definitively breaks latency (c0005). The residual gain available through this mechanism at 1-in-8 is real but modest; compounding it with other mechanisms has not yet succeeded cleanly.
- **Admission gate / frequency threshold:** Stricter thresholds reliably raise hit rate in isolation but reliably disrupt cache residency at a cost that swamps the gain (c0004 v003, c0005 v003). A path to making this mechanism viable requires decoupling residency effects from threshold tightening — most plausibly by limiting its firing rate, not by softening the threshold value itself.
- **LIRS-trace workload character:** Stable hot set, Zipfian distribution, long-lived frequency signal. Any mechanism that resets accumulated counts (decay, aggressive admission turnover) has a structural disadvantage. Latency is sensitive to cache-line residency composition; even correct-direction hit-rate changes can raise p50/p99 if they alter which nodes are hot.
