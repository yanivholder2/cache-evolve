# Cycle 0003 Insight

## Cohort outcome

All three variants built on cycle 0002's v001. v001 moved that policy onto the O(1) index/slab structure without changing behaviour. v002 replaced the occupancy gate with a threshold that switches between 1 and 2 based on a windowed ghost-return rate. v003 gave keys returning from ghost one second-chance credit when they re-enter main. v001 won because it was the only variant that didn't regress: its hit-rate matched the parent bit for bit, and both policy variants held or fell in every cell.

## Patterns

- The ghost path doesn't make a good lever in either direction. Using ghost returns as a signal to loosen admission (v002) and rewarding returnees with credit (v003) both lost hit-rate. Under steady-state pressure the ghost mostly holds weak single-reuse keys that v001 refused to graduate, so a ghost return is weak evidence of value.
- v002's switch fired on the wrong traces. It turned lenient on web_0 and on hm_0 at the large size, where strictness had helped, and gained nothing on hm_0 at the small size. Its ratio mixes up how often keys return over a long horizon with what strictness actually costs. The ghost is about 3× larger than the small queue, which inflates the ratio.
- Main stays full, so any extra protection given to one population evicts someone else. v003's credit kept dead keys in main for an extra lap and pushed out entries that would have been hit.
- Across three cycles, the baseline strict two-reuse rule plus the freq reset has held up every time. None of the threshold, split, gate or credit changes has beaten it on all traces.
- Per-op-time and metadata-size have now been missing from the evaluation for three cycles in a row. The cost refactor has been checked for correctness (an oracle test enforces identical hit/miss behaviour), but nobody knows whether it is actually faster.

## Open questions

- How much does v001 reduce per-op-time, and how much does it increase metadata-size, compared with its linear-scan parent?
- Where did v002's threshold settle on each trace? Its threshold-stats output was never captured.
- Would a gate that directly compares the misses avoided by promoting against the main entries displaced by promoting do better than any rate-based proxy?

## Continuation

Use v001 (the O(1) index with the occupancy-gated two-reuse rule) as the parent. Before running more hypotheses, make sure the evaluation measures per-op-time and metadata-size. Drop ghost-rate threshold switching and flat ghost-return credit. If policy work continues, condition it on evidence each key carries with it (for example, its small-queue freq when it went to ghost), or compare the cost of promoting a key against the cost of displacing one directly. Otherwise, accept that the policy has plateaued and spend hypotheses on reducing cost and metadata.
