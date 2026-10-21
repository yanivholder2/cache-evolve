# Cycle 0009 Insight

## Cohort outcome

Three probes were tried on the 4x-wide split: mid-probation insertion (v001), skipping the protected-MRU rotate for hits already in the top quarter (v002), and a larger slots/8 probation fraction on wide buckets (v003). v002 was the only Winner, and only as an exact hit-rate tie with its parent. v001 lost slightly, and v003 beat the baseline but was not compared against the incumbent.

## Patterns

- Insertion position is now negative at both extremes. Victim-end insertion (cycle 0008) and mid-probation insertion (this cycle) both cost hits, because new keys need residency to earn a second reference. Insertion point is not a useful knob.
- The Winner is a speed-motivated refactor with identical hit-rate, so it wins by default. Its goal, lower per-op-time, is again unmeasured.
- v003 repeats the pattern of a lean positional split beating the baseline. It does not show that slots/8 beats the incumbent's slots/16, so the probation fraction question is still open.
- Per-op-time and metadata-size are missing for the ninth cycle running. The cost of 4x-wide scans, segment shuffles and skip-move checks has never been priced.
- Authors again could not read the overlay code, so their explanations rest on the hypothesis text and are inferred. An exact tie may also mean the edit had no effect.

## Open questions

- Does the skip-move rule lower per-op-time enough to justify keeping it?
- Does the 4x width, with its marginal hit-rate gain, survive pricing in time?
- Is the hit-rate ceiling for this design on this trace reached, so that further gains need a different policy family?
- Do the results hold on other traces?

## Continuation

Stop probing hit-rate knobs in the segmented split: insertion point (victim-end or mid-probation), probation fraction, associativity, promotion and demotion. Get per-op-time and metadata-size into the fitness vector before another cycle, so v002's skip-move and the wide buckets can be judged. After that, either try a different policy family such as frequency-aware admission, or run robustness checks on other traces.
