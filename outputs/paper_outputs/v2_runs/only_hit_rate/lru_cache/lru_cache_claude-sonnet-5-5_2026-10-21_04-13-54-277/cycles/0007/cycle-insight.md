# Cycle 0007 Insight

## Cohort outcome

Three variants were tried: a frequency-aware victim scan over the oldest probation entries, a much longer sketch aging window with a decoupled doorkeeper reset, and a simplified rebuild of the segmented LRU with a sketch gate and no doorkeeper, starting from baseline. Only the simplified rebuild (v003) won. The victim scan was marginal at best, and the longer aging window lost a little hit-rate.

## Patterns

- Hit-rate was again the only measured dimension. The winner's claimed savings in per-op-time and metadata-size are unverified, so it is unclear whether it won on hit-rate or only on a more compact design. This is the seventh cycle with the cost metrics missing.
- Slower sketch decay hurt. Stale high-count keys block newer candidates at admission, so the aging window should stay near its current length. The test also changed the doorkeeper reset, so the two effects cannot be separated.
- Frequency-aware victim selection now has a real signal because frequency persists across segments. The gain is small, which suggests the probation LRU head was already close to the best victim.
- The simplest design (SLRU plus a sketch gate that counts rejected misses, no doorkeeper) was competitive. This is consistent with earlier cycles, where the doorkeeper's gain was small and offset by its costs.
- All differences are marginal and rest on one trace at one cache size.

## Open questions

- Does the doorkeeper-free design hold hit-rate while being cheaper? That cannot be known until per-op-time and metadata-size are measured.
- Does the victim scan add per-insert cost that outweighs its small gain?
- Would a smaller aging-window step, or testing the window and the doorkeeper reset separately, show anything?
- Is the missing cost measurement a harness fault?

## Continuation

Stop lengthening the aging window and stop widening the victim scan; both are at a plateau on this trace. Treat the doorkeeper-free SLRU plus sketch (v003) as the simplicity baseline. Fix the metadata-size and per-op-time measurement and add a second trace or cache size before any more hypotheses. After that, try a different axis, such as adaptive segment sizing, or a cost-aware comparison of v003 against the doorkeeper variants.
