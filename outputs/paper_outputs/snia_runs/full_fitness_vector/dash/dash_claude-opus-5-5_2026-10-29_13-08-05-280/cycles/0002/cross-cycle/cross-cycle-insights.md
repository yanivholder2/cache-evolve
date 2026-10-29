# Cross-Cycle Insights

## Recurring wins

- The stash reuse gate holds up as the main lever for hit-rate. It was the strongest winner in 0001, and in 0002 its plain two-hit threshold turned out to carry almost all of the gain. Admitting or keeping items based on evidence of reuse is the only direction that has raised hit-rate.
- Cutting multi-bucket moves (promotions plus demotion cascades) improves both hit-rate and per-op time. Changes that remove churn win, and changes that add churn lose even when each insert gets cheaper.
- A lean common hit path pays off in median get latency. Removing per-hit counter writes and comparisons recovered the median get cost the gate had added in 0001.
- Large cache sizes are where the hit-rate is won, with hm_0 the clearest case. Once churn stops there is room for a reused working set.

## Recurring failures

- Placement based only on free space (balanced placement, and later a version limited by a reuse flag) never moved hit-rate in either cycle. Its latency gain from 0001 is now in doubt.
- Adding further "protect reused items" mechanisms on top of the gate (MRU demotion landing, reuse flags) is mostly redundant. They brought small or no gains because the gate already captures that reuse signal.
- Any bookkeeping on the target-hit path (counter reads or writes, comparisons, conditional stores) steadily costs median get. That cost often eats the benefit of the change.
- MRU landing for promotions adds promotion churn and slows ops. This holds because insert position and promotion speed are coupled through the hottest-in-target → probing rule.

## Open questions

- How noisy is the per-op-time measurement? Re-running an identical baseline is needed before small latency differences are trusted. That includes past latency "wins", which were judged on this measurement.
- Can a write-free victim guard (a single bit, or one active only at large sizes) win back the large-cache hit-rate lost when the victim comparison was dropped?
- What causes the put p99 rise under the threshold-only gate? Is it stash churn from unconditional promotion into a full target?
- What limits web_0 at the large size, if placement does not?
- Should the gate threshold grow with cache size?
- Would requiring reuse before target → probing promotion cut churn without hurting hit-rate?

## Per-policy notes

- Dash segment (stash / target / probing tiers): every variant so far changed promotion or placement within the segment and left Item layout alone. The current baseline is the threshold-only stash gate with a lean hit path (cycle 0002, v001).
- Gate parts priced separately: the threshold gives the hit-rate. The per-hit counter writes cause the median get cost. The victim comparison gives a small hit-rate gain at large sizes, mainly on hm_0, for a small get cost.
- Free-slot balanced placement: retire it as a hit-rate strategy. Keep it only for latency, and only after the noise check, with free slots kept for hot items rather than cold evictees.
- MRU landing: promotion landing hurts through churn. Demotion-only landing gives at most a small gain, at web_0 large, and is redundant with the gate.
- Untried: Item layout changes, and gating promotion from target to probing.
