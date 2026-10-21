## Recurring wins

- Changing which keys are admitted or how hits are treated beats changing where keys are placed. MRU placement on hit (cycle 1) and the second-sighting admission filter (cycle 3) are the only improvements so far.
- Gains come from protecting recently useful entries and filtering one-hit-wonders, not from moving more items.
- Each win built on the prior incumbent (cycle 1 v001, then the admission filter). Branching from the winner is a sound strategy.

## Recurring failures

- Placement-position tweaks beyond the first one (insert path, cold-end demotion, eligibility relaxation) gave equal or marginally better hit-rate. They look exhausted on workload-a.
- Extra movement (bidirectional swaps, more promotions) causes demotion cascades. Delaying or relocating eviction (wider probe neighbourhood) lets cold entries crowd out hot ones.
- Several variants matched their parent's hit-rate exactly (cycles 2 and 3). Overlay edits may be dead code, or the workload may be insensitive. This ambiguity wastes cycles.
- Only hit-rate has been measured in every cycle. Per-op-time and metadata-size are unevaluated, so no cost claims exist.
- Authors cannot read overlay sources, so all mechanism claims are inferred.

## Open questions

- Do the wins generalize to other workloads and cache sizes? Everything so far is workload-a.
- What do the admission filter's memory and miss-path hashing cost? Does the gain survive once per-op-time and metadata-size are in the fitness vector?
- Is the admission threshold tuned well? Would a frequency-based or doorkeeper-style signal do better?
- Are the edits that reproduced parent hit-rate exactly (especially victim selection) on the live path? Re-test victim selection only once that is confirmed.
- Does combining the admission filter with MRU placement add anything beyond what each gives alone?

## Per-policy notes

- Dash segment (bucket, probing, stash) with promotion on hit: MRU placement on promotion helps. Eligibility and back-promotion changes do not.
- Insert and demotion paths: position changes were no-ops on workload-a.
- Victim selection (furthest-from-MRU scan): neutral, with unverified liveness.
- Admission (second-sighting filter): the best lever so far. Next step is to tune it and try doorkeeper or frequency variants.
- Probe structure (wider neighbourhood before the stash): slightly negative. Drop it.
