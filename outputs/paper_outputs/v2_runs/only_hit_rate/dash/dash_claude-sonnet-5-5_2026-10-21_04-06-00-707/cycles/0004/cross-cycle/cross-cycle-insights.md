## Recurring wins

- Changes that decide *what is admitted or promoted* paid off; changes that decide *where things sit* mostly did not.
- Two wins so far: MRU placement on hit-promotion (cycle 1) and an unconditional second-sighting admission filter (cycle 3). Both are simple, low-machinery changes.
- Both wins rest on one idea: protect recently useful entries and keep one-hit-wonders out.

## Recurring failures

- Placement-position tweaks (insert, demotion, eligibility) added nothing beyond the cycle 1 win. Several reproduced the parent's hit-rate exactly.
- Adding movement (bidirectional swaps, more promotions, wider probe neighbourhoods) caused cascades or let cold entries crowd out hot ones.
- Refining the admission filter (counters with aging, victim-conditional exemption, doorkeeper bit filters) lost hit-rate every time. Smaller or faster-resetting state raised collisions and forgot once-seen keys. "Victim never hit" is a weak value signal.
- Exact-tie results recur (cycles 2 and 3). The overlay may not be on the executed path, and authors cannot read overlay sources, so these null results are ambiguous.
- "Winner" labels can mean "best of a weak cohort" or "matched the parent", not a real improvement (cycles 2 and 4).

## Open questions

- Do the wins generalize beyond workload-a and the current cache size? Every conclusion comes from a single trace.
- What do the wins cost? Hit-rate is the only dimension ever reported. Per-op-time and metadata-size are unmeasured, which matters for the filter's per-segment state and miss-path hashing.
- Are the exact-tie variants live code? Is victim selection still untested because its edit never ran?
- Is the winner rule relative to the cohort? Or is the fitness vector missing dimensions?
- Untried levers: bucket and probing structure, and a frequency signal that the eviction decision actually uses.

## Per-policy notes

- Current incumbent: cycle 3 v002 (one-bit unconditional second-sighting filter), built on the cycle 1 v001 MRU promotion. It appears to be a local optimum for admission rules.
- Promotion/placement: MRU on hit is good. Further placement variants are exhausted on workload-a.
- Victim selection: the furthest-from-MRU scan tied its parent. This is inconclusive until it is confirmed live.
- Probing/stash: widening the neighbourhood cost a little hit-rate. Stash-eligibility relaxation was noise.
- Admission: the simple one-bit filter beat all the more elaborate variants.
