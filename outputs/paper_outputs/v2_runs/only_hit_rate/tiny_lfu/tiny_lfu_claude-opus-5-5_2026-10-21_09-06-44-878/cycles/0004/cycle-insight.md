# Cycle 0004 Insight

## Cohort outcome

All three variants built on the cycle-0003 wide sketch and each changed one thing about what the estimates mean. v001 deduplicated the get-miss-then-put double count. v002 added conservative update. v003 added very-long-period halving aging. Only v001 won, by a small margin. The other two were flat, within noise.

## Patterns

- Making estimates more accurate has stopped paying off. On the 16×-wide sketch, conservative update made no difference, because collision overcount is already gone from the minimum row. Width and conservative update fix the same error, so their gains don't add up.
- Aging is now settled as negative for workload-a. It still didn't help on a clean sketch at a very long period. This confirms that the earlier "stale-high" blocking came from collisions, not from genuinely old popularity. Lifetime frequency is the right signal for this trace.
- Making counts honest helped, but only a little. With dedup, sketch counts match true access counts, so keys that keep missing no longer outrank residents that collect counts through hits. But a first-time newcomer still ties a resident that was accessed once, and the newcomer loses that tie.
- The sketch side (width, update rule, aging, dedup) now looks used up as a hit-rate lever. Over four cycles, the only large gain came from removing collision noise.
- This is the fourth cycle in a row where per-op-time and metadata-size went unscored. v001 should lower per-op cost (one hash per get, and fewer increments), but that has not been measured.

## Open questions

- Is the remaining hit-rate in the resident structure instead? For example, a small window or doorkeeper in front of the main region, or an SLRU probation/protected split, now that estimates are clean and honest. These were only tested earlier on the noisy sketch.
- How much can the sketch shrink before hit-rate drops, now that conservative update can make up for collisions at a narrower width? This matters once metadata-size is priced.

## Continuation

Build on v001 (wide sketch with dedup) as the new baseline. Drop further work on sketch accuracy, all aging, tie-admission and wider victim sampling. Next, re-test structural levers on the clean estimates, such as an SLRU-style protected/probation split or a small fixed window. If metadata-size is finally scored, run a narrower-sketch-plus-conservative-update test to recover memory. Keep demanding per-op-time and metadata-size reporting.
