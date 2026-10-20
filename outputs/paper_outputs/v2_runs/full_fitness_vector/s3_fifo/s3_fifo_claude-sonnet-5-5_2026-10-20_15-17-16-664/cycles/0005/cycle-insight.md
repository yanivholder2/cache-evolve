# Cycle 0005 Insight

## Cohort outcome

Three independent experiments from different parents: an O(1) ring-buffer ghost (v001), a CLOCK-style pop/push rewrite of main eviction (v002), and a 5% small queue from baseline (v003). Only v001 has a mechanism for its gain (lower `put` cost at unchanged hit rate). v003 shows a modest hit-rate gain, but its large latency drop is implausible for a one-constant edit. v002 was neutral and probably won on noise.

## Patterns

- Both behavior-preserving rewrites (v001, v002) kept hit rate and eviction decisions identical. Only the one that targeted ghost maintenance (v001) moved `put`. The other, on main eviction (v002), did not.
- Together these confirm that the `put` tail comes from ghost maintenance, not from main-queue eviction or membership checks.
- The per-op-time headline tracks `get` p50, so a variant that improves `put` can still look worse on the headline (v001). `get` shifts appeared in variants that never touched `get` (v001, v002), so they are noise.
- The 5% small ratio now has a second hit-rate gain on workload-a, from a clean baseline. It is consistent in direction but still small, and it is single-trace.
- The latency drop in v003 has no mechanism and should not be credited to the ratio change.
- Whole-cache footprint grows slightly with each new structure (ring bookkeeping here), and the per-entry `metadata-size` metric does not show it.

## Open questions

- Does the 5% small-ratio gain survive repeated runs and other traces and sizes?
- Does v001's ring ghost combine with the index and scan work from earlier cycles without a `get` regression?
- What is left in the `put` tail now that ghost trimming is O(1)? Candidates are hash-map operations and the small-to-main move.
- Is the `get` p50 shift real or a layout effect?

## Continuation

Adopt v001's ring-buffer ghost as the base for `put` work, and stop reworking main-queue eviction (v002 shows nothing to gain). Treat 5% as the small ratio and stop sweeping it unless repeated runs on other traces disagree. Next, re-measure `get` p50 with repeated runs, and look for the remaining `put` tail cost outside ghost trimming and eviction. Judge on hit rate and on latency changes that have a clear mechanism.
