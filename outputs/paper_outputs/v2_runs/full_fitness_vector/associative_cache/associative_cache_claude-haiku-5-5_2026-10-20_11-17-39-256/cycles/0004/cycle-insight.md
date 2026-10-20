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
