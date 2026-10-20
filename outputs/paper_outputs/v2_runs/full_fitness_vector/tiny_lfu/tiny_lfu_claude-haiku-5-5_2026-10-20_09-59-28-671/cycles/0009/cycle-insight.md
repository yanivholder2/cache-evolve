# Cycle 0009 Insight

## Cohort outcome

The cohort tried three changes: collapsing the parent's two-pass conservative sketch update into one loop that reuses admission-check counts (v001), inlining the packed protected-flag accessors on the get path (v002), and narrowing sketch counters to 8 bits to fit more columns in the same byte budget (v003). Only v003 showed a hit-rate movement, and it was small; v001 and v002 left hit rate flat.

## Patterns

- Hit rate moved in only one variant. The narrower-counter, wider-sketch layout (v003) nudged hit rate up slightly, which fits the Cycle 0008 collision-spread direction. Its saturating counters also add a possible admission-resolution cost that was not isolated.
- The single-loop conservative update (v001) improved tail latency for both get and put, which is the first sign that the extra per-increment pass flagged in Cycle 0008 can be removed without losing the collision benefit. The put median went the wrong way, so the gain is not yet clean.
- Inlining the get-side flag accessors (v002) did not produce a get-side gain, and put timings rose in the same run. An edit that should not touch the put path moved it, which points to run noise or an overlay that differs from the description.
- Timing signals are weak across the cohort. Each comes from a single run, and the sources were not inspected, so no timing result here should be read as a speed win.
- Metadata stayed flat in all three variants, so the byte budget held under the column-for-width trade.

## Open questions

- Does the v003 hit-rate gain hold on a second trace and at other cache sizes, and does saturation at narrow counter width cost admission accuracy?
- Does the v001 tail improvement survive repeat runs, and what drives its put median regression?
- Is the v002 get and put slowdown noise, or does the overlay differ from the described change?
- Are the v003 timing drops real, given the hypothesis made no timing claim?

## Continuation

Pursue the v003 counter-width and column trade: test it on another trace or size, and check whether narrow saturating counters hurt admission before building further on it. Keep the single-loop conservative update from v001 as the preferred way to get collision spread without the extra pass, but require repeat runs that clear the put median before accepting it. Drop the get-side accessor inlining in v002 as a speed direction on this workload.
