# Cycle 0003 Insight

## Cohort outcome

The cohort tried three changes against the frequency-based parent from Cycle 0002: caching the minimum-counter slot for faster victim lookup, merging the victim search with a per-eviction counter halving, and a second-chance reference-bit scheme for victim selection. Only the second-chance variant (v003) landed as a Winner, gaining hit-rate on workload-a and lowering put cost, at the price of slightly higher get cost and larger per-item metadata.

## Patterns

- Both frequency-line variants that tried to speed up the put-side victim search (cached minimum, merged scan with halving) failed to help. Each one either slowed the put path or lost hit-rate, so the put-side cost from Cycle 0002 is not solved by these mechanisms.
- The cached-minimum shortcut was undone by its own maintenance: counter increments on get hits keep invalidating the cached slot, so puts still fall back to the full scan and pay the bookkeeping cost on top.
- Adding a miss-driven decay on top of the existing periodic halving erased the frequency protection too quickly. Aging is already bounded by the periodic schedule, so extra decay is the wrong lever.
- Re-reference-aware victim selection through a second-chance bit retained re-referenced keys through one-shot bursts, the same pollution pattern the frequency work targeted. It gained hit-rate and reduced put cost, which is the first cohort result that improved both at once.
- The Winner's per-item metadata grew, and the get path picked up a small extra write per hit. Metadata growth suggests more state than the single bit the hypothesis specified.
- The Winner's mechanism is inferred, not confirmed: its source overlay was not readable this cycle, so the attribution rests on the hypothesis, fitness vectors, and metadata delta.

## Open questions

- Is the second-chance gain real and mechanism-driven, or does it come from something else in the overlay? The overlay needs to be checked before the attribution is relied on.
- How many bits of per-item state does the Winner actually use, and can the metadata growth be cut back to a single bit without losing the hit-rate gain?
- Does the second-chance gain hold on traces other than workload-a?
- Can the get-side write on every hit be avoided, for example by setting the reference state only on some hits?

## Continuation

Next cycle should build on the second-chance reference scheme: confirm the mechanism, tighten the per-item state toward one bit, and cut the get-side write before adding anything else. Drop the cached-minimum shortcut and the per-eviction halving as axes; neither has a path to a win on the current policy. Keep the periodic halving schedule as is.
</content>
