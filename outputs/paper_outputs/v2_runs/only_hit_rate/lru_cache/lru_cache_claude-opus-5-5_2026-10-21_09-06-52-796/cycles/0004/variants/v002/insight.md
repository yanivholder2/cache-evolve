# Variant v002 Insight

## Helped

(No measured dimension improved. The `ghost_set` sequence-number fix stops a stale `ghost_q` copy from erasing a newer ghost entry. That is a correctness fix, but the hit-rate result shows it did not offset the admission changes made alongside it.)

## Hurt

- hit-rate: Every new key now enters `small`, so a ghost hit no longer puts a key into `main`. It only lowers that key's promotion bar to one hit. Cold keys now need two hits (`PROMOTE_FREQ = 2`). Both hits have to land while the key sits in a probation FIFO that holds only `capacity / 10` entries. So keys whose reuse gap is a bit longer than that short window are now evicted. Under the parent, a single hit or a ghost hit would have kept them, and they would have gone on to earn hits in `main`.
- hit-rate (secondary effect): More keys now fail the bar in `small`, so a larger share of evictions come from `small` and get pushed into the ghost. Re-admitted ghosted keys that miss their one-hit bar are ghosted again too. The ghost FIFO therefore turns over faster per request. That shortens the effective re-reference window that v001 had just tuned with `ghost_cap = capacity / 2`.

## Takeaway

The hypothesis wanted every route into `main` to require proven reuse while resident. On workload-a that test is too strict, because the probation queue is tiny: a ghost hit was a useful signal of reuse, and requiring a further resident hit inside `small` throws away re-references the ghost had correctly caught. This variant made three changes at once: ghost routing, the two-hit bar for cold keys, and the ghost bookkeeping fix. So the regression can't be pinned on one of them. A cleaner next step is to keep the bookkeeping fix, restore direct ghost-to-`main` admission, and test each admission change on its own.
