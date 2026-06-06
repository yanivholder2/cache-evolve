# Cycle 0009 Insight

## Cohort outcome

This cycle attacked the one named, never-directly-tried hot-path cost —
per-hit link surgery — by replacing recency tracking with CLOCK
second-chance bits, from two placements, plus a footprint-shrink control.
The breakthrough was v003: a clean single-list CLOCK rewritten from
baseline cut both get and put latency, collapsed `Node` to 24 bytes by
dropping the doubly-linked recency list, and held hit-rate at/above exact
LRU. Grafting the same CLOCK bit onto the SLRU protected segment (v001)
regressed, and the struct-narrowing probe (v002) bought nothing — neither
advanced. v003 opens a fresh lineage that displaces the long-standing SLRU
champion.

## Patterns

- Placement, not the CLOCK mechanism, decided the outcome. Bolting a
  reference bit onto the SLRU protected segment (v001) backfired because
  the parent's protected-MRU branch was already write-free; CLOCK
  reintroduced a store on the hottest hits and taxed every link touch with
  mask/preserve. The same mechanism applied to a single flat list (v003),
  where there is no write-free MRU fast path to lose, turned every hit into
  one boolean write and won. The surgery the prior cycles flagged as
  "still holding room" was only being paid on the single-list structure.
- The headline cross-cycle assumption is now overturned: exact LRU recency
  tracking buys no hit-rate on this short-reuse trace. A single CLOCK bit
  matches LRU's hit-rate while removing the recency list entirely — cutting
  latency and per-node footprint together. The pointer surgery the
  segmented champion paid was pure overhead for this workload.
- The footprint-to-p50 coupling that haunted prior cycles is refuted as
  causal on the cache struct: shrinking the control block to `u32` fields
  (v002) left put flat and nudged get up. The earlier footprint
  perturbations were not a symmetric, exploitable lever on `LruCache`.

## Open questions

- v003 abandons segmentation wholesale — does the SLRU protected/probation
  split add anything over flat CLOCK on this trace, or was the entire
  segmented lineage carrying cost without payoff here?
- CLOCK's hit-rate edged above exact LRU; is that noise, or does the
  second-chance approximation genuinely suit this short-reuse pattern in a
  way strict recency does not?

## Continuation

Promote v003 (single-list FastHasher CLOCK, 24-byte node, no recency list)
as the new base — it is the first frontier mover in several cycles and
beats the SLRU champion on latency and footprint at equal hit-rate. Pursue
the flat-CLOCK lineage next: tune the clock (hand placement, sweep bound,
admission of fresh keys) rather than reaching back toward segmentation.
Drop CLOCK-on-SLRU (the protected fast path is already write-free) and
control-struct narrowing (footprint coupling is not a causal latency
lever). The standing "champion is at the frontier" conclusion no longer
holds — a structural change reached room the SLRU edits never could.
