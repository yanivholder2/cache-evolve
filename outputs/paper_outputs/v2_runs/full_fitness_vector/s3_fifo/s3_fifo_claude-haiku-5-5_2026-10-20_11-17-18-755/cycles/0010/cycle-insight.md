# Cycle 0010 Insight

## Cohort outcome

All three variants edited the ghost structure: a batched ghost prune with a half-capacity cap, a tombstone-based ghost table at a lower load factor, and an epoch-cleared bit array in place of exact ghost keys. All three are labeled Winners, but the gains are on the put path only. Hit rate held or rose slightly, and every variant grew the container, with the ghost's heap storage unmeasured.

## Patterns

- Ghost-side work is the whole cohort. Put-path gains came from removing or deferring per-insert ghost work: batched expiry (v001), tombstone deletion with a sized table (v002), and bit-array membership with no per-key bookkeeping (v003).
- Hit rate is essentially flat. The batched cap left it exactly at the parent (v001), and the tombstone table did too (v002). Only the bit array moved, slightly upward (v003), despite its false positives. The cap's small gain from Cycle 0009 did not reproduce once expiry was batched.
- Container footprint went up in all three, and none of the cohort's measurements can see the heap-side ghost storage. The footprint claims from Cycles 0007 through 0009 remain unresolved, and the Winner labels here rest on the put side alone.
- v002 is a direct follow-up to the Cycle 0009 flat table's put-tail regression. Its tombstones and lower load factor addressed the probe-work cost that Cycle 0009 left open, though it bundles two changes, so the cause is not isolated.
- v003's get-path improvement is not explained by its change. Per the shared-noise pattern from earlier cycles, it should not be credited as a get-path effect.

## Open questions

- Does the tombstone table's put-tail gain survive a measurement of its heap table bytes, and does it hold when tombstones and load factor are separated?
- Does the bit array's hit-rate uptick hold on other traces and capacities, and is its false-positive rate at a level the admission rule tolerates?
- Does the batched cap's hit-rate effect stay at zero on other traces, or was Cycle 0009's small gain specific to the per-insert drain?
- Do the tombstone table and the bit array conflict, since both replace the same ghost storage? They were not tested together.
- Why were all three labeled Winners when none shows a container-size or hit-rate gain that the evidence clearly supports?

## Continuation

Next cycle should pursue the ghost storage replacement as the live direction, with the tombstone table (v002) as the lead put-path candidate, but only after its heap table bytes are measured. Test the bit array (v003) on more traces and capacities before crediting its hit-rate uptick, and rerun its get-path baseline before crediting any get effect. Drop the batched cap as a hit-rate lever on current evidence, though batched expiry itself is worth keeping as a put-path tool. Do not count any of this cohort as a Winner on the container-size reading.
