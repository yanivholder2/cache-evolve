# Cycle 0007 Insight

## Cohort outcome

Three changes were tried: widening the sketch rows 4x (v001), counting each access once instead of twice on a miss-then-put (v002), and choosing the colder of the two oldest probation entries as victim (v003). v001 was the Winner with a small hit-rate gain in the predicted direction. It is the first width change to help, after several cycles of narrowing that lost. v002 lost slightly and v003 was a no-op within noise.

## Patterns

- Wider rows helped slightly, and narrower rows lost in cycles 4 and 5. Width therefore moves hit-rate monotonically but with sharply diminishing returns. The sketch is only mildly collision-limited.
- The Winner's gain is small and is bought with a metadata cost that is still unmeasured. Its value as a Winner depends on a size measurement that does not exist.
- The double count on a get-miss followed by a put acts as a useful admission bias toward new keys. Removing it made the unchanged `>= 2` tie-admit threshold too strict. Count semantics and the threshold are coupled and cannot be tuned independently.
- Victim selection inside probation (sampling two) is flat, like the earlier segment-internal tweaks of cycles 4 and 6. Probation order is mostly recency order, so the second-oldest entry is rarely colder.
- For the seventh cycle running, the fitness vectors report hit-rate only. The costs of v001 (size) and v003 (extra sketch estimates per miss) are invisible, and variant authors again could not read overlay source.

## Open questions

- Is v001's hit-rate gain worth four times the sketch metadata? An intermediate width such as 8x might capture most of it.
- Would the single-count change work if the tie-admit threshold were retuned together with it (for example to `>= 1`)?
- Does the never-decaying sketch limit hit-rate, so that aging or reset period is the real lever?
- Is workload-a saturated, so that only small gains remain?

## Continuation

Fix the evaluation first: add metadata-size and per-op-time to the fitness output, plus a second trace (shifting popularity or a different cache size). Then judge v001's width against its size cost, trying an intermediate width. Pursue sketch aging or reset-period changes, and retune count semantics and the tie-admit threshold together if revisiting single-counting. Drop probation victim sampling, further segment-internal tuning, and standalone single-count changes.
