# Cycle 0009 Insight

## Cohort outcome

All three variants attacked the single live dimension flagged by prior
cycles — scored metadata-size — branching off both O(1)-ghost families
(separate `ghost_set` and folded-tombstone). Two ported the redundant-key
deletion across designs to reach the same shrunken `Entry`; one went
further and relocated the freq counter out of the struct entirely. All
three were Winners, hit-rate bit-identical, latency untouched.

## Patterns

- The redundant `key` field generalized cleanly across both surviving
  ghost designs: dropping it (carried as zero-sized `PhantomData<K>`)
  shrinks the scored `Entry` identically whether ghosts live in a separate
  set or as in-map tombstones. The field was dead weight in every design
  because the resident key already lives in the `entries` map key and the
  order queues — confirming the metadata win is design-independent.
- The apparent metadata floor was alignment padding, not a hard limit.
  After the key drop, `value`+`freq` padded up under `u64` alignment;
  relocating `freq` into the map's value slot beside a value-only `Entry`
  collapsed the scored type below that floor with no change to real
  per-slot memory or hot paths. Scored size is now driveable by field
  *relocation*, not just deletion.
- Every move was measurement-only: no eviction, promotion, or ghost
  probe was touched, so hit-rate and latency held exactly. The cohort
  confirms metadata-size is fully decoupled from the closed hit-rate and
  scan-latency frontiers.

## Open questions

- Now that scored `Entry` is driven to a single aligned word by relocating
  fields off it, does the scored metric still reflect real memory — or has
  the cohort begun optimizing the *measurement* (padding, field location)
  rather than actual footprint?
- The container-memory frontier (separate ghost set vs. tombstone-laden
  single map) was named but not contested this cycle. Is it still the next
  binding constraint, and does freq-relocation shift the real per-slot
  layout in either family's favor?
- With hit-rate, both scans, and now scored `Entry` size all at floors, is
  any scored axis left with genuine headroom?

## Continuation

Field deletion *and* relocation have now exhausted the scored `Entry` type
— it is down to a single word and the "floor" was shown to be padding, so
stop chasing further per-entry shrinkage; the remaining slack is
measurement artifact, not memory. Pivot the next cycle to the unscored
container-memory frontier that has been flagged for three cycles running
but never directly tested: reconcile the separate-`ghost_set` vs.
folded-tombstone families on real footprint, since they make opposite
tradeoffs for identical scored results. Treat hit-rate, scan-latency, and
scored metadata-size all as closed (bit-identical / floored). If no scored
axis has real headroom left, the honest next question is whether the
scoring captures the real cost the container designs differ on.
