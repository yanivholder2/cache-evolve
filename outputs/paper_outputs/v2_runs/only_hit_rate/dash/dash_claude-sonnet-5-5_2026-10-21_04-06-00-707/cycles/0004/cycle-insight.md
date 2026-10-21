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
