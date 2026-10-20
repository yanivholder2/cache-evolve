# Cycle 0009 Insight

## Cohort outcome

The cohort tried storing 64-bit fingerprints in the ghost queue (v001), feeding main-queue evictions at zero frequency into the ghost queue (v002), and collapsing the residency probe and insert in `put` into one entry call (v003). Only v002 is a Winner, with a small hit-rate gain on lirs workload-a at capacity 2048; v001 and v003 are hit-rate neutral and carry only unmeasured cost claims.

## Patterns

- v002 is the only variant with a measured hit-rate movement, and it is the first ghost-side change this line has produced a gain from. It changes which keys reach the ghost queue, not the ghost window, small-queue routing, or re-admission frequency, which earlier cycles tried and dropped. The gain is marginal and rests on one trace at one size.
- v001 and v003 held hit-rate identical to the parent, as their hypotheses predicted. Both are constant-factor or representation changes. Neither reports per-op-time or metadata-size, so their savings cannot be credited.
- v001 adds a second key hash on every ghost check and push while leaving the linear ghost scan and `retain` in place, so the miss-path cost may not improve at all.
- v003 calls `key.clone()` on every `put`, including hits, which may cancel the saving from the single hash for heap-allocated keys.
- v002 leaves the ghost `retain` and `any` scans unchanged, so it adds ghost entries without changing the cost of ghost lookups.
- Overlay edits were not independently confirmed for any variant this cycle. The v002 gain should be tied to its edit in the built code before it is treated as real.
- Evidence is still one trace (lirs workload-a) at one capacity (2048).

## Open questions

- Does the v002 gain hold on other traces and capacities, or is it specific to lirs workload-a at 2048?
- Does the v002 gain persist once ghost entries from main evictions are weighed against the cost of the unchanged ghost scans?
- Do the v001 fingerprint and the v003 single-hash `put` lower per-op-time or metadata-size once the extra hash and the `key.clone()` are counted?

## Continuation

Carry v002 forward as the lead hit-rate candidate: pursue main-eviction ghost feeding, and require a rerun on additional traces and at least one other capacity, plus confirmation of its overlay edit. Keep v001 and v003 only as cost candidates, and drop them as standalone changes unless a rerun reports per-op-time and metadata-size. Do not spend the next cycle on small-queue or ghost-window tweaks beyond the main-eviction feed.
