# Task: cycle-insight-writer

Cycle: 0005
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0005

## Winners

- v001
- v002
- v003

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0005/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0005/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0005/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0005/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0005/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0005/hypotheses/H03.md

## Prior cycles digest

### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

The cohort tried a fingerprint prefilter on bucket lookups, a longer bucket-local ghost ring, and frequency-based victim selection using per-slot access counters. Only the frequency-based victim selection (v003) landed as a Winner, and it gained hit-rate on workload-a at a cost in per-op time.

## Patterns

- Frequency signals helped hit-rate where recency-only ordering had not. Re-referenced keys survived bursts of one-shot inserts, which is the pollution pattern the bucket-local work has been chasing.
- The gain came with a victim search that scans the full bucket on put, so the cost sits on the hot path. The per-op regression is the main thing the Winner has to fix.
- Lengthening the ghost ring did not lift hit-rate. This closes the question cycle 0001 left open about whether the ring was simply too short, and points to the reinsertion idea itself as the weak part.
- Adding a fingerprint prefilter to 32-slot buckets added a second linear scan on get and extra write work on every put. Hit-rate was unchanged, so the regression is isolated to the layer itself.
- Metadata-size reported no change in any variant, including ones that clearly add per-bucket or per-slot state. The size probe does not appear to count the new state, so it should not be used to judge storage cost this cycle.
- Get and put latency movements were single-run and in some cases not explained by the diffs. Treat the size of any timing gain as uncertain until re-measured.

## Open questions

- Can the frequency victim search be made cheaper (tracking a running minimum, or fewer ways) without losing the hit-rate gain?
- Does the frequency gain hold on traces other than workload-a, or is it tied to its one-shot insert pattern?
- What does the size probe need to count so that per-bucket and per-slot state shows up in metadata-size?

## Continuation

Next cycle should build on frequency-based victim selection and attack its put-side cost first, keeping hit-rate fixed. Drop the fingerprint prefilter and ghost-ring length as axes. Do not tune ghost-ring length further.


### Cycle 0003

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


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

The cohort tried three changes on the second-chance and frequency line: moving the reference bit from each item into a per-bucket mask (v001), narrowing the frequency victim search to a rotating window (v002), and replacing the reference bit with a 2-bit re-reference value per slot (v003). All three landed as Winners, but they win on different axes: v001 saves metadata at a time cost, v002 buys put latency at a hit-rate cost, and v003 gains hit-rate with a put-tail cost.

## Patterns

- Retention signals keep helping hit-rate on workload-a when they separate re-referenced items from one-shot inserts. Both the per-bucket mask (v001, same hit-rate as parent) and the multi-level re-reference value (v003) preserve or extend that effect.
- Put-side cost is the common thread. Shrinking the victim scan (v002) reduced put latency but weakened the frequency protection, which suggests the full-bucket view on eviction is what carries the hit-rate gain.
- Moving reference state out of the item and into the bucket (v001) traded memory for time: get and put both slowed, so per-bucket state is not free on either path.
- The multi-level re-reference value (v003) pushed the put tail up through its aging rounds, so the policy's cost has shifted from the get-side write toward the bounded aging scan on full-bucket puts.
- Taken together with Cycle 0003, reference state is moving in two directions at once: v001 pushes toward fewer bits per item, while v003 adds bits per slot for more levels. The cohort has not yet shown which direction is worth its cost.
- The mechanisms for the put and get timing shifts are inferred from diffs and fitness vectors, not confirmed from overlay code, because the overlays could not be read this cycle. Treat the timing attributions as provisional.

## Open questions

- Is the per-bucket mask's time regression a fixed cost of the bucket-word write, or can the bit-scan in the clock walk be made cheap enough to keep the metadata saving?
- Does the 2-bit re-reference gain hold on traces other than workload-a and at other cache sizes?
- Can the put-tail cost of the aging rounds be cut by capping rounds or by tracking the aging state, without giving back the hit-rate?
- Does narrowing the victim search lose hit-rate on other traces too, or is the loss specific to workload-a's one-shot pattern?

## Continuation

Next cycle should pursue the re-reference value (v003) and attack its put-tail cost first, keeping hit-rate fixed. Keep the per-bucket mask (v001) only as a metadata option, and only if a follow-up can recover its get and put time. Drop the rotating-window victim search (v002) as a put-side fix; cheaper bookkeeping for the true global minimum is the better direction.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
