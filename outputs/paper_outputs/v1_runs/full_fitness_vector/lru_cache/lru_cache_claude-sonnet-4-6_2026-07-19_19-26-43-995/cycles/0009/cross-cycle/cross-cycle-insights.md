## Recurring wins

- **Node metadata compaction** reliably wins: u32 sentinel links (cycle 0001) and the link-array split (cycle 0005) each halved or quartered node size without adding per-op branches. Both landed as clean winners with zero latency regression.
- **Two-queue policy via spare-bit encoding**: folding the queue-membership flag into the existing `prev` pointer kept Node layout unchanged while unlocking full 2Q semantics (cycle 0002). Reusing existing fields at zero layout cost is the most reliable pattern for policy additions.
- **87.5% protected quota (capacity/8 probationary)**: confirmed cost-free on multiple independently-baselined parents across cycles 0006–0009. It consistently improves hit-rate without measurable latency regression and is the settled policy ceiling.
- **Simple structural changes outperform complex algorithmic ones**: every cycle where a mechanism added branching or pointer indirection regressed; every pure layout change either won or did no harm. The workload's tight per-op budget is unforgiving to conditional logic.
- **FNV-1a hasher**: confirmed genuine struct-size reduction and lower hash computation cost (cycle 0009); not yet tested in isolation at the clean baseline — a likely latency win still pending.

## Recurring failures

- **CLOCK / sweep-based eviction**: failed in cycle 0001 (hand-sweep cache-line pollution) and cycle 0004 (Vec header bloat plus same sweep cost at demotion). Amortised relinking is unfavorable when promotions and demotions are rate-coupled at steady state.
- **SEEN_BIT / two-hit admission gate**: tested across cycles 0004, 0005, and 0006 in three distinct implementations (splice-on-first-hit, cost-free bit-OR, eviction-rescue). All failed for the same root cause: workload-a does not generate enough within-window repeat accesses for any two-hit requirement to be net beneficial.
- **Quota above 87.5%**: cycles 0007, 0008, and 0009 all confirmed that any step beyond capacity/8 probationary carries a genuine per-op-time regression. Three independent remedies — intermediate quotas, demotion batching, FNV-1a offset — each failed to break the coupling.
- **Adaptive mechanisms on a stationary workload**: adaptive quota sizing (cycle 0004) and CountMin sketch graduation gate (cycle 0007) both added overhead for logic that nearly never fired. The Zipfian hot set is stable enough to defeat any mechanism designed for phase shifts.
- **Demotion batching**: defeated at high quotas because the tighter probationary window simultaneously raises graduation frequency, leaving net demotion burden unchanged or worse (cycle 0009).
- **Forced inlining / `entry()` API tricks**: `#[inline(always)]` displaced hot-path code from instruction cache; the `entry()` API's borrow-checker constraint forced an extra pointer move on the dominant eviction path (cycle 0002). Both backfired.
- **MRU-tail guard composition**: won as a standalone on its original workload (cycle 0002) but failed when composed with 2Q (cycle 0003) — fire rate too low to amortise the branch cost on a diffuse access pattern.

## Open questions

- **FNV-1a at the clean 87.5% baseline**: the hasher speedup has only ever been tested paired with a quota increase. Tested alone, it may lower the per-op-time floor and reopen a small window for future quota experiments.
- **Node cache-line alignment / arena packing**: the 16-byte link-array node (cycle 0005) did not show a measurable latency benefit, but the gain is in principle real. At the current higher hit-rate from quota wins, does the layout dividend finally surface above the measurement floor?
- **Lazy-promotion segmented LRU**: a fundamentally different policy structure that might achieve a similar effective protected ratio without eager per-operation demotion enforcement — the mechanism behind the confirmed latency cost.
- **Frequency sketch as within-probationary eviction ordering**: using a sketch to evict estimated low-frequency items first (rather than strict FIFO) within the probationary queue remains untested; it avoids the graduation-gate failure mode that closed the sketch-as-filter branch.
- **Minimum viable probationary pool diversity**: at 6.25% (capacity/16) the probationary budget may be too narrow to replenish the protected queue with sufficient item diversity; the reversal point has not been directly confirmed.

## Per-policy notes

- **LRU (baseline)**: structurally optimised to a 16-byte node via two successive compaction wins. Per-op latency is tight enough that any extra branch shows up immediately; further gains likely require layout or hash-lookup improvements, not behavioral changes.
- **Two-queue (2Q) with protected/probationary split**: the dominant policy direction across all cycles. Best confirmed quota: 87.5% protected. Quota compression above that carries a real latency cost confirmed from three independent angles. The policy suits the stable Zipfian hot set well; adaptive variants add overhead for no benefit.
- **CLOCK**: conclusively closed. Both the original sweep variant (cycle 0001) and the bitmap-backed deferred variant (cycle 0004) failed; the underlying trade-off — amortised relinking only pays when promotions vastly outnumber demotions — is not met at steady state.
- **Two-hit / second-chance admission**: conclusively closed on this workload. Tested across four cycles in every plausible implementation; the workload simply does not produce the repeat-access density these mechanisms require.
- **Frequency sketch**: as a graduation gate, found no discrimination signal on this workload (cycle 0007). As an eviction-ordering heuristic within probationary, untested and still open.
