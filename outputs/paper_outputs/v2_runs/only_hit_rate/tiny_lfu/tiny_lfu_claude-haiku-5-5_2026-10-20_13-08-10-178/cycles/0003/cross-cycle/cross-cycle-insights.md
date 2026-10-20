## Recurring wins

- Layout-only changes to the frequency sketch have not regressed hit-rate in any cycle so far (Cycles 0001–0003). They are safe to keep, but no hit-rate gain has been shown from them.
- Restructuring recency order inside the main region (probation/protected split) is the only change so far to move hit-rate upward (Cycle 0003). It leaves the sketch and admission path untouched.

## Recurring failures

- Changes that alter sketch behaviour lose hit-rate: fixed-schedule halving (Cycle 0001) and a changed key-to-counter mapping (Cycle 0001, small dip). Decay discards admission signal at this capacity rather than removing stale history.
- Changes that move when or where admission sees a new key lose hit-rate: a recency window ahead of the LRU with admission at window exit (Cycle 0002). The one-touch-burst benefit it targeted did not show up on this trace.
- Claimed gains from layout work (memory footprint, per-op time) keep landing on dimensions the fitness vector does not measure, so those Winners advance only by not regressing.

## Open questions

- Do layout-only Winners gain anything once a memory or timing dimension is in the fitness vector, and do they justify keeping them?
- Does the probation/protected gain hold on other traces and cache sizes, or is it specific to the current trace?
- Does the sketch's saturation point bind on very hot keys, creating admission ties?
- Would a gentler or staleness-gated decay recover signal without the admission losses seen so far?
- Is the small hit-rate dip from the key-to-counter mapping change within noise?

## Per-policy notes

- TinyLFU-style admission (frequency sketch plus admission comparison): every sketch-side or admission-timing change so far has cost hit-rate. Keep sketch counts and the admission rule fixed; treat layout work here as hit-rate-neutral only.
- Window-in-front-of-LRU: tried once (Cycle 0002), lost hit-rate. Do not re-propose at this capacity.
- Main-region LRU (segmented into probation and protected): the only hit-rate gain so far (Cycle 0003). Provisional until tested beyond a single trace and size; build on it before adding other changes.
- Across the ledger, all cycles have used one eviction-policy family and one trace at one cache size, so policy-level conclusions are narrow.
