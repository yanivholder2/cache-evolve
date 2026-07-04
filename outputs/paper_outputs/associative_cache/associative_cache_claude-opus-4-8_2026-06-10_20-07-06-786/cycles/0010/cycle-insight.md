# Cycle 0010 Insight

## Cohort outcome

Two slots consolidated the prior cycle's cost rewrites — v001 stacked the
u64-tick metadata shrink and a rotate-replace put path onto the geometry
parent, and v002 made the surviving all-protected full-bucket branch O(1) —
both byte-identical and both winning as undifferentiated hit-rate ties. The
third slot (v003) broke from the cost-only pattern with a behavioural
front-admission "evict newest unpromoted" move; it did not win.

## Patterns

- The two winners answer the standing composition question affirmatively: the
  independent cost rewrites stack to a byte-identical cache state with no
  hidden interaction, and the full-bucket put path is now O(1) end to end. The
  payoff still lives entirely on the size and per-op-time axes the hit-rate
  fitness cannot see, so they land as ties — the fifth straight cycle of this.
- v003 reopened the supposedly exhausted behavioural search with a
  scan-resistant probation idea, but it did not displace the byte-identical
  baseline. Whatever hit-rate it claimed did not clear the bar, and it carries
  a real O(bucket_size) put cost — a behavioural gamble that cost more than it
  returned under the only axis being measured.
- The cohort confirms the loop is fully in cost-consolidation mode: the
  selectable wins are provably-free rewrites, and the one behavioural probe
  could not beat them.

## Open questions

- Did v003's front-admission actually move hit-rate at all, or was the
  scan-resistance claim not borne out on this trace? Worth a clean re-test
  before that idea is recycled.
- The metadata shrink and O(1) put are now both in the baseline — does any
  combination of them register on a fitness vector that still sees only
  hit-rate, or do they stay collectively invisible?

## Continuation

Adopt v001 and v002 as the baseline — byte-equivalent, cost-favourable, and
together they complete the O(1) put path and the smaller `Item` footprint.
Do not recycle v003's front-admission as a hit-rate play without first
confirming it moves the metric; under the current fitness it only adds put
cost. The decision deferred for five cycles is now unavoidable: either add a
per-op-time / metadata-size fitness dimension so these recurring cost wins
become selectable, or declare the trace solved and stop. A hit-rate-only
fitness can no longer distinguish anything this cohort produces.
