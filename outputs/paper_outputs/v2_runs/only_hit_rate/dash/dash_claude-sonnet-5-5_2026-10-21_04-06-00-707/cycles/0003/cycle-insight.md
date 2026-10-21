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
