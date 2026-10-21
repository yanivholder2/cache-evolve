# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants tweaked promotion logic in `DashSegment::get`/`promotion_get` against baseline: insertion position (v001), promotion eligibility into probing (v002), and probing→target back-promotion (v003). Only v001, the one-argument change to MRU insertion, won.

## Patterns

- Only hit-rate was measured. Metadata-size and per-op-time are unevaluated for every variant, so no cost claims exist yet.
- The winning lever changed where a promoted item lands, not how many items move. Variants that added movement (v002 more promotions, v003 bidirectional swaps) gained little or lost.
- Extra movement tends to cause demotion cascades toward stash and eviction. This offsets or exceeds the benefit of relocating items.
- The recency-skew rationale (don't make just-hit items the next victim) was supported by v001. Spare probing capacity was not a binding constraint (v002, marginal gain).

## Open questions

- Does MRU placement generalize beyond workload-a, and does it cost per-op time?
- Would combining v001 with v002 add anything, or is v002's marginal gain noise?
- Do other insertion positions (demotion path, stash→target vs. probing retries separately) matter?

## Continuation

Build on v001: explore insertion-position variants on other paths (demotion, cascade to stash) and test across more workloads, ideally with per-op-time and metadata in the fitness vector. Drop bidirectional probing→target swaps (v003) and don't expect more from eligibility relaxation alone (v002).


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

Three placement/eligibility tweaks were tried: MRU placement on the insert path (v001, parent cycle-1 v001), stash→probing promotion eligibility stacked on the MRU winner (v002, same parent), and cold-end demotion of displaced victims from baseline (v003). Hit-rate was unchanged from the parent in all three. v001 and v002 are listed as Winners, but only by matching the incumbent, not by improving on it.

## Patterns

- Every variant reproduced its parent's hit-rate exactly. That points to no-ops on workload-a at this cache size, or to overlays that never changed the executed path. It does not show the levers are harmful.
- The insert-path and eligibility levers add nothing beyond the hit-promotion position change. This supports the view that cycle 1's marginal eligibility gain was noise or already subsumed by MRU promotion.
- The cascade-eviction theory behind cold-end demotion got no support. Demotion position did not change which useful entries were lost.
- Only hit-rate was measured again. Metadata-size and per-op-time are still unevaluated, so there are no cost claims.
- Authors could not read the overlay sources, so the "overlay may not have taken effect" explanation could not be ruled out.

## Open questions

- Do these overlays actually alter the executed code path, or are they dead edits?
- Is workload-a or the cache size insensitive to placement on insert and demotion paths? Other traces or sizes might separate them.
- Does any variant differ on per-op-time or metadata, even with equal hit-rate?

## Continuation

The placement-position family (insert, demotion, eligibility) looks exhausted on workload-a. Drop further variants of it unless they are checked against a different workload or cache size. Next cycle should use a different lever, such as eviction victim selection, bucket and probing structure, or admission, branching from cycle/0001/v001. It should also add per-op-time and metadata-size to the fitness vector so ties can be broken.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

Three new levers were tried: victim selection by a furthest-from-MRU scan (v001), a second-sighting admission filter on the insert path (v002), and a wider probe neighbourhood to adjacent buckets before the stash (v003). Only the admission filter (v002) won, improving hit-rate on workload-a. Victim selection left hit-rate unchanged, and the wider probe lowered it slightly.

## Patterns

- The win came from deciding which keys get admitted. Cycles 1 and 2 changed where keys are placed, and that did not help. This points to one-hit-wonders as the main source of waste on this trace.
- Levers that delay or relocate eviction did not help. The wider probe let colder entries stay resident and crowd out hotter ones, which cost a little hit-rate.
- v001 reproduced its parent's hit-rate exactly, as the cycle 2 variants did. The edited victim-selection code may not be on the executed path, so this result is not clean evidence against eviction policy.
- Only hit-rate was reported again. The costs of v002's filter state and miss-path hashing, and of v003's extra probes, remain unmeasured.
- Authors still could not read the overlay sources, so every mechanism claim is inferred.

## Open questions

- Does the admission gain hold on other workloads and cache sizes? Is it worth the per-segment filter memory and the extra work on each miss?
- Is the filter's "full enough to need eviction" threshold tuned well, and would a different admission signal do better?
- Was the victim-selection edit live? Does a victim-selection change that provably runs behave differently?

## Continuation

Branch from v002 and keep pursuing admission: tune the threshold and filter design, and try frequency-based or doorkeeper-style variants. Drop wider-probe neighbourhoods (v003) and further placement tweaks. Put per-op-time and metadata-size in the fitness vector before the next cycle, so v002's gain can be weighed against its cost. Re-test victim selection only after confirming the edit is on a live code path.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

All three variants refined the cycle 3 second-sighting admission filter: a 2-bit saturating counter with aging (v001), a victim-conditional filter that admits first sightings when the cold-end victim was never hit (v002), and a small periodically-reset doorkeeper bit filter on the eviction path (v003). None improved hit-rate on workload-a. v002 is listed as Winner, but its own insight reports a hit-rate drop below the parent. It looks like the least-bad variant, not a real gain.

## Patterns

- Every refinement of the parent's unconditional one-bit filter lost hit-rate. The parent's design looks like a local optimum, and extra machinery added noise.
- Filter accuracy mattered more than filter sophistication. Smaller or faster-resetting state (halved slots, 256-bit filter wiped at half fill) raised collisions and forgot once-seen keys, so reusable keys were rejected again.
- A single "victim never hit" bit is a poor value signal. Un-hit entries are often still warm, and exempting them from the filter let one-hit-wonders back in.
- Only hit-rate was reported again. The metadata and per-op-time costs of the extra filter state and miss-path work remain unmeasured.
- Authors could not read the overlay sources, so the mechanisms are inferred.

## Open questions

- Is the parent's admission gain robust on other workloads and cache sizes, or specific to workload-a?
- Would a counter or doorkeeper pay off if the threshold or the eviction decision actually used the extra frequency information, with filter size kept constant?
- Why was a variant that lost hit-rate selected as a Winner? Is the winner rule relative to the cohort, or is the fitness vector missing dimensions?

## Continuation

Treat the cycle 3 v002 one-bit unconditional filter as the incumbent and stop tweaking its admission rule, victim exemption and repacked counters included. Next, explore a different lever: bucket and probing structure, or victim selection confirmed to be on a live code path. Alternatively, test the incumbent on other workloads and sizes. Add per-op-time and metadata-size to the fitness vector before any further filter work.


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
