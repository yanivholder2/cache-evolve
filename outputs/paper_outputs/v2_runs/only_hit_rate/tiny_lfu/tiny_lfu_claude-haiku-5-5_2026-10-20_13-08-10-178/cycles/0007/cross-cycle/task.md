# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

The cohort tried one sketch-decay change, one layout-only narrowing of node and index types, and one change to how the sketch derives counter positions from a key. The only Winner, v002, was the layout-only change, which left admission and the sketch untouched; the two variants that altered sketch behaviour did not hold up on hit-rate.

## Patterns

- Changes that touched the frequency sketch or the admission comparison were the ones that lost hit-rate; both sketch-side variants moved admission estimates in the wrong direction.
- Decay of the sketch on a fixed schedule discarded signal that admission needed at this window size, rather than removing stale history.
- The Winner's payoff is not yet demonstrated: its fitness reflects only hit-rate, which it kept flat, so its intended metadata-size gain is unmeasured.
- None of the variant insights recorded a hit-rate gain, so this cycle's Winner advanced by not regressing rather than by improving.

## Open questions

- Does the per-entry size reduction from the Winner produce a measurable benefit once a metadata or memory dimension is part of the fitness vector?
- Is the small hit-rate dip from the key-to-counter mapping change within noise, or a real regression that should rule out that family of changes?
- Would a gentler or staleness-gated decay of the sketch recover signal without the admission losses seen here?

## Continuation

Next cycle, pursue changes that leave the sketch and admission rule intact and target dimensions the fitness vector actually measures, and add a measurable metadata or memory signal before judging layout work. Drop plain fixed-schedule sketch halving at this window size, and do not re-run key-hashing rewrites unless they are shown to be within noise.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

The cohort tried two layout-only changes to the frequency sketch's storage (narrower counters, and a flattened row-major counter array) and one structural change adding a small recency window ahead of the main LRU with admission applied at window exit. Both Winners, v001 and v003, were layout changes that left hit-rate flat; the window variant, v002, lost a little hit-rate and did not advance.

## Patterns

- Layout-only changes to the sketch were the only ones that held hit-rate. Both Winners preserve the logical counter mapping, so their hit-rate is unchanged by construction rather than by a measured improvement.
- Changing when and where admission sees a new key cost hit-rate on this trace; the window's benefit for one-touch bursts was not visible.
- None of the cohort recorded a hit-rate gain. Every benefit claimed by the Winners (memory footprint, per-op cost) sits on dimensions the fitness vector does not measure, so the Winners advance by not regressing.
- The unmeasured-dimension problem carried over from Cycle 0001 is unresolved: layout work keeps producing Winners whose payoff cannot be confirmed.

## Open questions

- Would a memory or per-operation timing dimension in the fitness vector confirm the layout gains, and do they justify keeping both Winners?
- Does the narrower counter type ever create admission ties on traces with very hot keys, where saturation could matter?
- Is there a window or admission structure that helps one-touch bursts without giving up the sketch signal at this capacity?

## Continuation

Next cycle, keep to changes that preserve sketch counts and the admission rule, and prefer layout or allocation work only if the fitness vector gains a metadata or timing signal that can confirm it. Drop the recency-window-in-front-of-LRU structure at this capacity, and do not stack further layout changes until the Winners' gains are measured.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

The cohort tried two sketch-layout changes (narrower counters, and hoisting per-key row indices into a shared buffer) and one change to the main region's internal ordering, splitting its LRU into probation and protected segments. The only Winner, v002, was the main-region reordering, the one change that showed a hit-rate movement; the two layout-only variants preserved behaviour but advanced on no measured dimension.

## Patterns

- The only hit-rate gain came from changing recency ordering inside the main region, with the sketch and admission path left alone. That is a different lever from the sketch-side and window-side changes of Cycles 0001 and 0002.
- Layout-only changes to the sketch again preserved hit-rate by construction, and again their claimed benefits (footprint, per-op time) sit on dimensions the fitness vector does not measure. This is the same unmeasured-payoff pattern seen in Cycle 0002.
- The Winner's signal rests on a single trace at a single cache size, so its gain is provisional rather than established.

## Open questions

- Does the probation/protected gain hold on other workloads and cache sizes, or is it specific to this trace?
- Would a timing or memory dimension in the fitness vector confirm the hashing-reuse and counter-narrowing gains, and justify keeping them?
- Does the sketch's saturation point ever bind on very hot keys, or does the admission comparison stay insensitive across workloads?

## Continuation

Next cycle, build on the main-region recency structure from v002: test it on additional workloads and sizes before stacking further changes on it, and avoid re-proposing the window-in-front-of-LRU idea. Drop further sketch-layout-only changes until the fitness vector gains a metadata or timing signal that can confirm them.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

The cohort tried a sketch-width change, a resizing of the protected segment within the main region, and a change to where demoted protected entries land in probation. The only Winner, v002, was the sketch-width change, the first sketch-side change in recent cycles to show a hit-rate gain; the two main-region variants both lost hit-rate against the parent.

## Patterns

- Widening the sketch reduced over-counting from colliding keys and produced the only hit-rate gain in the cohort. This contrasts with Cycle 0001, where sketch-side changes lost hit-rate; the difference is that this change addressed collision error rather than decay or key mapping.
- Both main-region variants moved away from the incumbent and both lost hit-rate. The current probation/protected split and the demotion placement at the back of probation both look like the better settings for this trace.
- The Winner's gain is small, and its predicted metadata cost (sketch size scaling with width) was not measured. Its advancement rests on hit-rate alone, the same unmeasured-cost pattern seen in Cycles 0002 and 0003.
- All three variants were evaluated on a single trace at a single cache size, so every result here is provisional.

## Open questions

- Does the width gain persist on other workloads and cache sizes, and is it worth its metadata cost once that cost is measured?
- Is there a point where further widening stops helping, i.e. where collision error is no longer the limiting factor in admission?
- Does the protected/probation ratio near 4/5 hold as the best setting on other traces, or is it specific to workload-a?

## Continuation

Next cycle, build on the sketch-width result by testing it on additional workloads and sizes and measuring its metadata footprint before stacking further sketch changes. Hold the current protected-segment ratio and demotion placement as-is, and drop the halved protected cap and the push-to-front demotion variants.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

The cohort tried a reduction in sketch row count, a conservative-update rule that raises only the counters at the row minimum, and periodic global aging of sketch counters. The only Winner, v002, was the conservative-update change, a modest hit-rate gain that came from tightening over-estimates rather than from any new memory.

## Patterns

- Both sketch-side changes that touched the estimator's error were the main levers this cycle. The Winner reduced collision over-counting by changing the update rule, which extends the collision-error lever first seen in Cycle 0004's width gain.
- Removing estimator depth hurt. The min-across-rows estimate depends on enough independent rows to suppress collisions, so the cohort's depth reduction undid the kind of gain that width provided.
- Aging the sketch on a fixed access-count trigger hurt. Discarding accumulated frequency history at this capacity weakened the admission comparison, so the sketch needs its history intact on this trace.
- The Winner's per-operation cost is unmeasured. Its extra minimum-row pass on every access is a plausible cost, and the fitness vector does not report timing to confirm it is acceptable.
- All results come from one trace at one cache size, so the Winner's gain is provisional.

## Open questions

- Does the conservative-update gain hold on other workloads and cache sizes, and what does its extra pass cost per operation?
- Is there an aging trigger, not tied only to access count, that keeps the admission comparison intact while still shedding stale frequency?
- Does conservative update stack cleanly with the Cycle 0004 width change, or do the two gains overlap?

## Continuation

Next cycle, build on the conservative-update rule from v002: test it on additional workloads and sizes, and pair it with a timing signal before stacking more estimator changes. Drop reductions in sketch depth, and avoid global halving on a fixed access-count trigger; any future aging idea should use a different trigger and be tested against the Winner.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

The cohort tried a conservative-update refinement on top of the wider sketch, a larger protected share within the main region, and a doorkeeper that gates first-touch keys out of the frequency sketch. The only Winner, v002, was the protected-share increase, a small hit-rate gain; the doorkeeper variant lost hit-rate and the conservative-update variant moved only slightly.

## Patterns

- Protected-segment sizing is the lever that moved in the Winner. It extends the Cycle 0003 finding that main-region recency structure matters, and it is the second time the protected/probation split has been the source of a gain after Cycle 0004 found the incumbent ratio was better than a halved cap.
- Estimator refinements are stacking with diminishing returns. The conservative-update rule gained only a little on top of the width change from Cycle 0004, which suggests the two act on overlapping collision error rather than independent sources.
- Filtering one-touch keys out of the sketch's update path hurt. The correction applied at estimate time does not fully replace the withheld increments, so the sketch's frequency history is still doing work that a gating shortcut loses.
- The Winner's attribution is not fully confirmed: its source change could not be inspected from the variant record, so the gain is credited to the segment split on the basis of the hypothesis and the fitness delta alone.
- Every result comes from one trace at one cache size, so all gains here are provisional.

## Open questions

- Does the larger protected share hold on other workloads and cache sizes, or does it trade away probation room that matters elsewhere?
- Does the conservative-update gain justify its per-operation cost once the width change is in place, and is the stacked benefit distinguishable from the width gain alone?
- Is there a doorkeeper-style estimate correction that matches the withheld sketch increments closely enough to be worth retrying?

## Continuation

Next cycle, confirm the protected-share gain on a second workload or cache size before building on it, and keep the width and conservative-update estimator changes as the baseline rather than stacking new estimator tweaks on the same over-counting error. Drop the doorkeeper-as-update-filter family in its current form.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

The cohort tried a deeper sketch (one more hash row), a larger protected share within the main region, and a promotion rule that gates probation-to-protected moves on the sketch estimate. Only one gain was reported, a slight hit-rate rise from the added sketch row (v002); the protected-share increase and the gated promotion both lost ground against the parent. The Winners list names v001, but v001's record reports a slight hit-rate loss, so the Winner designation needs checking before anything is built on it.

## Patterns

- Adding estimator depth helped where removing it hurt in Cycle 0005. Extra rows give the min-across-rows estimate more independent samples, which again points at collision error as the lever in the admission comparison.
- The protected share has a peak rather than a monotone trend. Cycle 0006 credited a larger protected share with a gain; pushing it further here reversed that, which suggests the useful range is close to the current setting and that probation room matters once the share gets large.
- Gating promotion on the sketch estimate hurt. Frequency counts lag behind recency, so the gate kept some entries in probation that were about to earn a second hit. Promotion on every hit appears to be the better rule on this trace.
- Depth's memory cost is unmeasured. The added row grows sketch counter storage in proportion, and no variant reports that footprint or any timing, so the gain cannot yet be weighed against its cost.
- Every result comes from one trace at one cache size, so all differences here are provisional.

## Open questions

- Is the Winners designation for v001 correct? Its record shows a hit-rate loss, while the only reported gain is v002's depth change.
- Does the depth gain hold on other workloads and sizes, and where does adding rows stop paying for the extra counter memory?
- Where is the protected-share optimum on other traces, and does it sit near the current ratio or somewhere between it and the reversed setting?

## Continuation

Next cycle, resolve the Winner attribution first: confirm whether v001 or v002 is the real advance before building on either. If the depth gain is kept, test it on a second workload or size and measure its counter memory before stacking further sketch changes. Hold the protected share at or near the incumbent ratio rather than pushing it higher, and drop the sketch-gated promotion rule.


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
