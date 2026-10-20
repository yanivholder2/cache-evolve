## Recurring wins

- Leaving the frequency sketch and the admission comparison untouched kept hit-rate flat, and that was the only non-regressing direction seen so far (one cycle of evidence).
- Layout-only changes to node and index types are a safe family to iterate on, though their benefit is not yet measured by the current fitness vector.

## Recurring failures

- Changes that alter the frequency sketch or the admission comparison lowered hit-rate in the cycle observed; both sketch-side variants moved admission estimates the wrong way.
- Fixed-schedule sketch decay discarded history that admission needed at this window size, rather than removing stale entries.
- Rewrites of how the sketch maps keys to counter positions produced a small hit-rate dip; not yet shown to be noise, so treat this family as suspect until cleared.

## Open questions

- Does a per-entry size reduction translate into a measurable gain once a metadata or memory dimension is added to the fitness vector?
- Is the small hit-rate dip from key-to-counter remapping within noise, or a real regression?
- Would a gentler or staleness-gated sketch decay keep useful signal without the admission losses seen with fixed-schedule halving?
- Can any variant show a hit-rate gain at all, rather than advancing only by not regressing?

## Per-policy notes

- TinyLFU (admission-gated, sketch-based): admission depends on the frequency sketch's estimates, so changes to sketch update, decay, or key mapping have been the high-risk area; layout changes that keep admission intact have been the safe area.
- No other eviction policies are represented in the ledger so far; the digest does not break results out per policy.
