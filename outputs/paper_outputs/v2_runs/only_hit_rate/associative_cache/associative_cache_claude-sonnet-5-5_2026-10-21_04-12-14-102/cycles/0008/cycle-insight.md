# Cycle 0008 Insight

## Cohort outcome

Three probes were tried: LIP-style insertion at the probation eviction end (v001), a further doubling of associativity to 4x baseline at fixed capacity with slots/16 probation (v002), and a boundary-insertion lean split with a skip-move rule for hits already in the recent quarter (v003). Only v002 was a Winner, with a marginal hit-rate gain. v001 lost hit-rate against its parent, and v003 improved on plain LRU but did not beat the incumbent.

## Patterns

- Insertion position is the one lever that moved things this cycle, in both directions. Inserting at the victim end starved new keys of the residency needed to earn a first re-reference. Inserting just above the probation boundary kept scan protection without that starvation. The trace's short reuse distances punish aggressive scan resistance.
- The associativity increment shrank again. The first doubling gave a clear gain, the second a marginal one, so the remaining misses look capacity- or recency-driven, not conflict-driven.
- v003 repeats the earlier finding that a lean positional split with no extra per-entry state beats plain LRU. Its comparison is against the baseline, not against the current best.
- The hit-rate levers on this split are now almost fully worked: size, promotion, demotion, ghost state, associativity and insertion point all give flat, marginal or negative results.
- For the eighth cycle running, only hit-rate is visible. The cost of wider scans in v002 and of positional checks in v003 is unpriced. Authors could not read the overlay code, so their explanations are inferred.

## Open questions

- Does the cost of 4x-wide bucket scans cancel the marginal v002 gain once per-op-time is measured?
- Is a mid-probation insertion point between MRU and the boundary better than either extreme, or is the parent's choice already near optimal?
- Is further gain only possible from a different policy family, such as frequency-aware admission, or does it need other workloads to show a difference?

## Continuation

Stop probing hit-rate knobs in the segmented split: associativity beyond the current width, LIP or victim-end insertion, probation size, promotion and demotion. Get per-op-time and metadata-size into the fitness vector first, then judge v002's width and v003's skip-move rule against them. After that, try a different policy family or robustness checks on other traces.
