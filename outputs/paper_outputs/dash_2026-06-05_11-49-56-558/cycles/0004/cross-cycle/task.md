# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants probed independent axes of the Dash promotion ladder — re-entry position, eviction policy, and bucket topology — and all three improved hit-rate with no regressions. The sole Winner, v003, won by rebalancing the normal/stash split to give the hot tier equal standing with cold storage.

## Patterns

- **Topology was the binding constraint.** v003 (50/50 normal/stash rebalance) won without touching any policy logic; simply allocating more hot-tier slots was the highest-leverage move, indicating the baseline over-provisions stash for this workload.
- **Two variants diagnosed the same underlying shortage from different angles.** v001 (MRU re-entry after stash hit) shortens the time an item spends below promotion eligibility; v003 directly expands the space items compete for once eligible. These axes are orthogonal and likely additive.
- **v002's LFU gain is workload-contingent.** The benefit was explicitly tied to workload-a's frequency-skewed access pattern; LRU's recency bias was harming a stable hot set. This is a narrower win than the structural fixes and may not generalise.
- **No variant hurt anything.** The baseline has multiple independent improvement opportunities that do not trade off against each other at this capacity.

## Open questions

- Do v001 (MRU re-entry) and v003 (topology) compound when applied together, or does one subsume the other?
- Is 50/50 the optimal normal/stash ratio for workload-a, or is there a higher-gain split beyond 16/16?
- Does LFU (v002) remain advantageous on less frequency-skewed traces, or does it regress on uniform/recency-heavy workloads?

## Continuation

The hot-tier capacity bottleneck is confirmed as the dominant lever for this cycle; future hypotheses should push the normal/stash ratio further (e.g., 20/12 or 24/8) to find the gain curve's peak or breakpoint. Combining MRU re-entry (v001) with the winning topology is a natural low-risk compound to test. LFU should be held in reserve for workload-specific probing rather than treated as a general improvement.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

This cycle probed three directions branching from Cycle 0001's winning topology: compounding MRU re-entry with the 50/50 split, pushing the split further to 75/25, and introducing aging-LFU from a fresh baseline. The sole Winner, v002, confirmed that the normal/stash gain curve had not yet peaked at 16/16 — expanding to 24 normal buckets produced a further lift with no regression, and stash can tolerate reduction to at least 8 slots without penalty.

## Patterns

- **Hot-tier expansion is still the dominant lever.** v002's 75/25 split won outright, reinforcing Cycle 0001's finding that topology is more impactful than policy changes. The working set fits comfortably within 24 normal buckets, meaning capacity was still the binding constraint after the 50/50 rebalance.
- **MRU re-entry and topology are additive but unequal.** v001's compound of MRU re-entry atop the 50/50 geometry improved on v003, confirming the two axes do not interfere, but the marginal gain was modest — once the hot tier is enlarged, collapsing the re-entry penalty matters less because items already have more dwell time.
- **Aggressive per-hit aging is harmful for this workload.** v003's symmetric decay (−1 to all co-resident items on each hit) drove mildly-warm items to eviction before they could contribute hits that recency-ordered LRU would have preserved. Workload-a's access distribution punishes coarse frequency discrimination more than it rewards it.

## Open questions

- Does the normal/stash gain curve continue past 24/8, or does a further push (e.g., 28/4) hit a breakpoint where stash under-capacity starts hurting cold-item re-entry?
- Would stacking MRU re-entry (v001's insertion-position change) onto v002's 75/25 topology produce further additive gains, as it did in Cycle 0001's compound test?
- Is there a gentler aging-LFU formulation — smaller decay increment or access-count-gated decay — that avoids draining mildly-warm items on workload-a?

## Continuation

The topology gain curve is still ascending; the next hypotheses-writer should probe 28/4 and possibly the theoretical limit of maximising normal buckets to locate the peak or breakpoint. Combining MRU re-entry with v002's 75/25 split is the natural low-risk compound to run in parallel. Aging-LFU in its current aggressive form should be dropped; any return to frequency-based eviction should use a finer-grained decay to avoid the mildly-warm eviction problem observed here.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

This cycle pushed three threads simultaneously: locating the ceiling of hot-tier expansion, compounding MRU re-entry onto the winning 75/25 topology, and retrying frequency-based eviction with a softer threshold gate. Only the compound (v002) won — with a small but clean gain — while the topology push exposed the breakpoint and the gentler aging-LFU still could not match position-based recency.

## Patterns

- **The topology gain curve has peaked.** Extending to 28/4 caused a regression, confirming that 24/8 is the optimal normal/stash split for workload-a. With only four stash slots, cold items cannot be held long enough to re-enter on their next access, and the added hot-tier pressure costs more than it returns. The geometry-only lever is now exhausted.
- **MRU re-entry is additive but nearly subsumed by topology.** Stacking Back-insert onto the 75/25 geometry produces a genuine, positive increment, but it is modest — because the enlarged normal-bucket pool already provides sufficient dwell time for natural promotion. The two axes do not interfere but they are not strongly synergistic either.
- **Position-based recency consistently beats frequency discrimination on this workload.** Threshold-gating the co-resident decay rescued co-residents from aggressive drain yet still underperformed ClassicLRU. The slow counter ramp-up creates its own eviction blind spot in early item life, replacing one failure mode with another. Item position encodes a stronger future-access signal than any frequency counter variant tested so far.

## Open questions

- With hot-tier topology exhausted and re-entry gains marginal, what genuinely new axes remain? Ghost/victim caches, admission control, or scan resistance have not been probed.
- Is there a re-entry formulation more nuanced than full Back-insert — e.g., inserting proportionally based on access count — that could extract more from the position lever?
- Would frequency-based eviction ever win on workload-a, or is the access pattern so recency-dominant that it is structurally incompatible?

## Continuation

The two previously dominant levers — hot-tier topology and MRU re-entry — are now close to fully exploited on workload-a; future hypotheses should pivot to unexplored structural axes such as ghost/victim-cache admission control or scan-resistant insertion policies. Frequency-based eviction in any form should be retired for this workload until evidence from a different workload trace motivates revisiting it. The confirmed optimal geometry (24/8) and the MRU re-entry policy together form the new compound baseline from which the next round of probes should launch.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

All three variants probed structural axes around stash admission and promotion gating — ghost-based bypass, LFU-gated promotion, and stash-free direct insertion — in line with Cycle 0003's call to explore unexplored terrain. Only v002 (ghost re-admission) won, and only by a margin that is effectively noise; the other two variants regressed.

## Patterns

- **The stash is a load-bearing quality gate, not just a capacity buffer.** Both attempts to weaken or circumvent it — front-door bypass on cold insertion (v003) and threshold-gating the hot-item promotion pipeline (v001) — degraded hit-rate. Removing or slowing stash traversal allows cold or slow-to-qualify items to displace established warm residents, which costs more than the saved hop recovers. This role as admission filter appears fundamental for workload-a's stable hot set.
- **Ghost re-admission is safe but near-neutral.** The ghost set (v002) produced only a marginal, noise-level improvement: the workload's reuse pattern does not cycle through the stash boundary frequently enough for bypass tracking to yield a real benefit. The mechanism is correct in principle but mismatched to this workload's access characteristics.
- **Promotion gating introduces friction with no real payoff.** The LFU threshold in v001 was designed to filter one-time scans from the probing tier, but scan pollution was not a meaningful loss source here — probing slots were already being used productively. Adding a counter-gated delay simply slows the hot-item pipeline.

## Open questions

- With topology, re-entry, admission control, and promotion gating all explored or exhausted, what genuinely novel levers remain for workload-a? Scan-resistant insertion *within* the normal tier (rather than at the stash boundary) has not been probed.
- Would any of these admission-control ideas produce real gains on a workload with heavier scan or bursty cold-item arrival patterns, where the quality-gate role of the stash is under more stress?
- Is the ghost set's near-zero gain truly a ceiling, or does it depend on the ghost set size and Front-insertion position chosen for re-admitted items?

## Continuation

The stash-as-admission-filter finding should be treated as a constraint: next-cycle hypotheses must not weaken or bypass stash traversal on new insertions. The ghost set (v002) can be set aside — it is safe but contributes nothing to workload-a. The improvement space for this workload is now quite narrow under current structural axes; the next hypotheses-writer should consider either intra-normal-tier eviction ordering (e.g., scan resistance or segmented LRU within the hot tier), or shifting to a second workload trace to reveal whether a different access pattern unlocks frequency-based or admission-control gains that are structurally blocked on workload-a.


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
