# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0007

## Winners

- v003

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0007/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

The cohort tried three different kinds of change: narrowing the full-bucket victim scan to unreferenced slots (v001), skipping the access-counter store on saturated get hits (v002), and adding a doorkeeper admission gate on the put-miss path (v003). Only the doorkeeper gate moved hit-rate; the other two changed timing alone, and none of the three improved the put tail.

## Patterns

- The admission gate (v003) is the first cohort change to act before eviction rather than inside it. It gains hit-rate on workload-a by keeping one-shot keys from reaching the victim scan, which is the pollution pattern the retention line has been chasing since Cycle 0003.
- The gate's costs sit outside what it targets: its storage, its periodic whole-structure clear, and an extra field on the cache struct all show up in metadata and in put tail and get timing, none of which the gate itself touches.
- Narrowing the victim scan (v001) did not shorten the put tail and slowed put. This is the second cycle in a row where a put-side shortcut to the scan has failed (see the rotating window in Cycle 0004), so the scan length is not where the put-side cost lives.
- The saturated-counter store skip (v002) showed a get-tail gain with hit-rate and metadata unchanged. The gain is concentrated in the tail, not the median, and put numbers moved with no put-path change, so single-run tails look noisy.
- The mechanisms are inferred from diffs and variant insights, not confirmed from overlay code, because overlays were not readable this cycle. Treat the attributions as provisional.

## Open questions

- Does the doorkeeper's hit-rate gain hold on traces other than workload-a and at other cache sizes?
- Can the gate's storage be made smaller and the whole-structure clear be spread out, so its put-tail cost goes away without giving back the admission effect?
- Is the get-tail gain from skipping the saturated-counter store real across repeated runs, or is it run-to-run variance?
- What is the actual source of the put tail on full-bucket puts? Cycles 0004 and 0005 blamed the aging rounds and the scan, and this cycle's scan change did not help.

## Continuation

Next cycle should pursue the doorkeeper admission gate (v003) as the main hit-rate lever, and attack its put-tail and metadata cost first, holding hit-rate fixed. Keep the saturated-counter store skip (v002) as a get-side candidate, but re-measure it with repeated runs before building on it. Drop the narrowed victim scan (v001) as a put-side fix.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
