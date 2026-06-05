# Cross-Cycle Insights

_Projection from ledger cycles 0001–0006. Regenerate from ledger; do not edit manually._

---

## Recurring wins

- **Hot-tier topology expansion** was the dominant lever through cycles 0001–0003. Shifting the normal/stash split from baseline toward 24/8 produced the largest cumulative gains; each increment confirmed the working set was still capacity-constrained until the breakpoint at 28/4.
- **MRU insertion position** has won or tied in every cycle where it was tested. Items inserted at the MRU end survive long enough to accumulate hits before eviction; any non-MRU entry position — probationary, LRU-end, demotion-at-LRU — consistently regresses on workload-a.
- **Eviction selectivity within the probing tier** emerged as the strongest new lever in cycle 0006. Preferring to evict items with zero in-tier accesses over confirmed-reuse residents directly addresses a real loss source that topology and position alone do not cover.
- **Stacking orthogonal axes** yields additive gains when the axes target different pipeline stages. Topology + MRU re-entry combined cleanly; frequency-guard eviction selectivity and second-chance stash recovery address distinct loss sites and are natural compounding candidates.
- **Second-chance stash recovery** (unconditional LRU-end promotion of stash-evicted items) produced a genuine, repeatable improvement over the ghost set by removing the dependency on a future external re-insertion event.

---

## Recurring failures

- **Frequency-based eviction (LFU, aging-LFU, threshold-gated LFU)** has never beaten position-based recency on workload-a across five cycles of reformulation. Coarse decay harms mildly-warm items; threshold gating creates an eviction blind spot during early item life; counter ramp-up is too slow for this stable hot set. Retire for workload-a unless a new trace motivates revisiting.
- **Ghost set re-admission** tested across multiple configurations (insertion positions, cycle 0002–0005) and consistently delivers noise-level movement. The workload's reuse pattern does not cycle frequently enough through the stash boundary for conditional bypass tracking to matter. Permanently retired.
- **Weakening or bypassing stash traversal** — whether via front-door bypass on cold insertions or LFU-gated promotion delays — consistently degraded hit-rate. The stash functions as a load-bearing admission filter on workload-a; any mechanism that shortcuts it allows cold items to displace warm residents.
- **Segmented-LRU promotion/demotion asymmetry** (promotions at MRU, demotions at LRU of destination) assumes a clean hot/cold divide that workload-a's uniformly warm distribution does not exhibit. Displaced items still carry residual heat; stripping their MRU position accelerates churn.
- **Pushing the normal/stash split past 24/8** hit a hard breakpoint at 28/4. With only four stash slots, cold items cannot be held long enough to re-enter, costing more than the added normal-bucket capacity returns. The topology lever is exhausted.

---

## Open questions

- Can the cycle-0006 frequency guard (evict zero-access items first) and second-chance stash recovery be stacked for an additive compound gain, given they target different pipeline stages?
- Is zero in-tier accesses the right eviction threshold, or does a threshold of one or two confirmed hits yield further improvement in the frequency guard?
- Do frequency-based or admission-control schemes become competitive on a second workload trace with heavier scan traffic or bursty cold-item arrivals — workload characteristics that workload-a's stable hot set structurally suppresses?
- Would a capacity stress test (smaller total cache) make probationary filtering or frequency discrimination genuinely pay off, where margin-case eviction decisions matter more?
- Are there structurally new axes — adaptive segment sizing, size-aware eviction, TTL-based decay, or scan-resistant insertion within the normal tier — that remain untested and could open new headroom?

---

## Per-policy notes

- **ClassicLRU / position-based recency:** The universal winner for workload-a. Item position encodes future-access signal more reliably than any frequency counter variant tested. MRU insertion at every tier boundary is consistently correct.
- **LFU / aging-LFU variants:** Structurally mismatched to workload-a's stable, recency-dominant access pattern. Coarse decay drains mildly-warm items; gated decay creates early-life blind spots. No formulation has been competitive across cycles 0001–0004.
- **Ghost set:** Confirmed noise-level across cycles 0002–0005 in all configurations. Mechanism is correct in principle but the workload does not generate enough stash-boundary reuse events for it to activate meaningfully. Retired.
- **Second-chance stash recovery:** Positive and repeatable (cycle 0006). Outperforms the ghost set because it is unconditional — recovery does not depend on a future external re-insertion event aligning with a ghost-set entry.
- **Frequency guard eviction selectivity:** Strongest single gain in recent cycles (cycle 0006 winner). Protecting confirmed-reuse residents from eviction by zero-access newcomers at the probing boundary is a real and unexploited loss site. The zero-access threshold and interaction with second-chance recovery remain to be probed.
- **TimestampLRU:** Real but modest gain in cycle 0006; the benefit of decoupling promotion candidacy from insertion order partially overlaps with gains already captured by the 24/8 topology and MRU-insertion combination, limiting its marginal contribution.
- **Segmented-LRU / promotion asymmetry:** Tested in cycle 0005, regressed. Textbook hot/cold divide assumptions do not hold on workload-a's uniformly warm distribution. Not a candidate for further probing under this workload.
