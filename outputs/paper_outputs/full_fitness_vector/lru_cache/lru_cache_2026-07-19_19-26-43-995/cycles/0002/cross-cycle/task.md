# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants targeted node memory compaction or hit-rate improvement via structural redesign; only v002's sentinel-based link compression landed as a clean winner, delivering smaller nodes and better get latency without sacrificing eviction quality.

## Patterns

- **Metadata compaction was the most reliable lever.** Both v002 (u32 sentinel links) and v003 (CLOCK flat slots) halved node size — but compaction that preserved the LRU chain (v002) also improved get latency through better cache-line utilization, while compaction that eliminated the chain (v003) introduced a new bottleneck via the CLOCK hand's sweep-induced cache-line pollution.
- **Algorithm complexity adds hidden per-op cost.** The two-queue variant (v001) and CLOCK (v003) both introduced extra pointer work or array traversal that eroded or negated their structural gains. The simpler structural change (v002) outperformed both.
- **Exact LRU ordering remains valuable for this workload.** v001's two-queue policy improved hit rate by protecting hot items, but v003's coarser single-bit recency signal hurt hit rate — suggesting the workload has enough reuse that precision in eviction decisions pays off.
- **Put latency is a secondary but real concern.** v002's slight put regression from cast overhead and v003's put blowup from CLOCK sweeps both appeared; get-path improvements should not be assumed to carry over to put.

## Open questions

- Can the two-queue hit-rate gain from v001 be recovered on top of v002's leaner node layout, where the extra per-queue bookkeeping fields represent a smaller fractional overhead?
- Is CLOCK's cache-line pollution inherent to the algorithm, or does a segmented or lazy-sweep variant recover the structural win without the latency penalty?
- How much further can node size be reduced (e.g., toward a single cache line per node) before diminishing returns set in?

## Continuation

v002's u32 sentinel-based links are the new structural baseline — next cycle should treat this as the parent and compound on it. The most promising directions are: (1) revisiting the two-queue policy on top of v002's leaner nodes to see whether hit-rate gains now come at an acceptable latency cost, and (2) exploring further struct packing to push node density higher. CLOCK-style approaches should be deprioritized unless a mechanism to avoid hand-sweep cache pollution is proposed.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

Two independent directions both landed as Winners: v001 layered a two-queue eviction policy onto the u32-sentinel baseline without growing the node, and v002 short-circuited the get promote path for already-MRU nodes. v003's attempt to accelerate put via the `entry()` API and forced inlining backfired entirely.

## Patterns

- **Policy and algorithmic shortcuts beat micro-optimization.** Both winners applied meaningful behavioral changes (2Q promotion logic, MRU-tail guard); the loser applied low-level code hints (`#[inline(always)]`, `entry()` API) that either couldn't reach their target path at steady state or expanded code size past the point where instruction-cache fit was preserved.
- **The protected-bit-in-prev encoding confirmed as practical.** Folding queue membership into a spare bit of the existing `prev` field kept Node at 24 bytes while unlocking full 2Q semantics — the cycle 0001 concern that 2Q required node growth turned out to be avoidable.
- **MRU-guard gains come with p99 and cross-path cost.** The get p50 win from skipping no-op promotes is real and large, but the added conditional disturbs branch-predictor and instruction-cache state enough to raise get p99 and put p50 — a reminder that single-path optimizations in hot mixed-workload code have side effects.
- **Forced inlining is not a free win.** `#[inline(always)]` on small helpers can push a hot path out of ideal instruction-cache placement, producing the opposite of the intended latency reduction.
- **Borrow-checker constraints cap `entry()` API gains.** On the eviction path (dominant at steady state), the occupied `VacantEntry` must be released before `map.remove()` can run, erasing the promised single-probe benefit and adding an extra pointer move instead.

## Open questions

- Can v001 (2Q) and v002 (MRU guard) be composed? The MRU guard would naturally extend to protected-queue tail hits; it may amplify the hit-rate benefit of 2Q by also cutting relinking cost on the most-recently-used protected entries.
- v002's p99 regression: is the branch-predictor disruption inherent to the guard, or can branch placement or profile-guided optimization recover the p50 gain without widening the tail?
- Is there a put-path latency improvement that does not run into the borrow-checker eviction-path obstacle? Pure structural changes (e.g., avoiding the key clone on eviction) may be safer than API-level tricks.

## Continuation

The next cycle should attempt to compose v001 and v002: apply the MRU-tail guard on top of the 2Q layout, since both are structurally compatible and target different bottlenecks (hit quality vs. promote cost on the hot path). The `#[inline(always)]` / `entry()` micro-optimization path should be dropped — the borrow-checker and I-cache constraints make it a poor fit. If a third variant is explored, focus on reducing key-clone cost during eviction rather than hash-probe count, as that is the remaining identifiable per-put cost not yet addressed.


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
