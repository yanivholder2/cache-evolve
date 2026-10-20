# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0006

## Winners

- v002
- v003
- v001

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0006/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

The cohort made three small, mostly put-side changes: an early exit in the frequency victim scan (v001), a word-level bucket scan that replaces the per-slot clock walk on top of the per-bucket mask (v002), and a bulk-add rewrite of the 2-bit re-reference aging (v003). All three kept hit-rate and metadata unchanged; the only movement was in timing, and it was mixed.

## Patterns

- Hit-rate held across the cohort, so these changes are behavior-preserving. Each one is an implementation-level rewrite of an existing mechanism, not a new retention policy.
- The per-bucket mask follow-up (v002) recovered put median time that the bit-by-bit clock walk had lost in Cycle 0004, which supports the idea that the scan form, not the mask itself, was the source of that regression. Put tail did not improve, though.
- The frequency early exit (v001) gave only a small put gain, which suggests a zero-counter slot is rarely present when a put scans the bucket. The scan usually runs to completion, so this shortcut is not where the put-side cost lives.
- The aging rewrite (v003) made the put path slower. The scan-then-bulk-add form appears to add full-bucket passes on the common full-bucket put, so the aging rounds were not the dominant put-tail cost. Cycle 0004 pointed at them; this cycle's result argues against that diagnosis.
- Get-side timing moved in v002 with no change to the get path, and the same kind of unexplained get movement appeared in earlier cycles. Treat get-side gains as noise until re-measured.
- Mechanisms here are inferred from diffs and variant insights, not confirmed from overlay code, because overlays were not readable this cycle.

## Open questions

- Where does the put tail actually come from on full-bucket puts, if neither the aging rounds nor the zero-counter scan dominates it?
- How often is a zero-counter slot present at put time, measured directly? This decides whether the early exit is worth keeping at all.
- Does the word-level scan's put-median gain hold across runs, and can its p99 rise be traced to the same code path?
- Is the get-side movement in v002 real or layout-related?

## Continuation

Next cycle should keep the per-bucket mask and the word-level scan (v002) as the put-side base, and profile the victim search on full-bucket puts before changing it again. Drop the aging rewrite (v003) as a put-tail fix; the scan-then-bulk-add form is a net loss. Drop the zero-counter early exit (v001) as an axis unless a measurement shows it fires often enough to matter.
</content>


## Write

Exactly one file: `cycle-insight.md` in the output dir.
