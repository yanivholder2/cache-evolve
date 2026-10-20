# Cycle 0002 Insight

## Cohort outcome

The cohort moved away from pure policy changes toward layout and bookkeeping: a deferred referenced-bit scheme with one bounded rescue, a capped second-chance scan, and narrower arena indices. All three Winners kept hit-rate roughly intact or gave up only a small amount, and each won on smaller nodes or cheaper operations rather than on hit-rate.

## Patterns

- Node-size reductions were the most reliable lever. Packing a flag into spare index bits and narrowing arena links both shrank the node, and neither cost hit-rate.
- Deferring recency work to eviction time helped the put path more than the get path. Removing relinks on hit did not measurably improve get, so the hit-path relink is not the dominant cost.
- Bounding the second-chance scan traded a small hit-rate loss for a predictable miss-path cost, and that tail was the clearest per-operation gain in the cohort.
- Narrowing indices saved memory but made get slower, which points at the width conversions on every arena access rather than at the policy. Memory wins and speed wins came apart here.
- Several attributions are inferred rather than verified. One variant's node-size result was larger than its hypothesis predicted, and one get-path change was not directly targeted, so those gains should not be credited to the mechanism named in the hypothesis until checked against the source.

## Open questions

- Is the get regression from narrowed indices caused by the casts at hot call sites, and can they be removed while keeping the smaller links?
- Does the referenced-bit scheme's hit-rate loss on re-referenced keys shrink if the single rescue is replaced with a different recency approximation?
- Were the get-path and node-size gains in the bounded-scan variant caused by layout effects or by measurement noise?

## Continuation

Next cycle, combine the cheap node-size reductions with the bounded eviction scan, and pursue ways to remove the index-conversion cost on get without giving back the smaller links. Keep avoiding list surgery on the hit path, since get did not respond to it. Drop further hit-path relink removal as a goal on its own, and avoid trading more hit-rate for put-tail gains unless the trade is clearly measured.
