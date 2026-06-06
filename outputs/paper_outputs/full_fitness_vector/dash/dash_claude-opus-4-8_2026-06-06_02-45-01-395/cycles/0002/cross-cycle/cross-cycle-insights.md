# Cross-Cycle Insights

## Recurring wins

- Latency wins have come repeatedly from deleting hidden hot-path cost
  (front-shifts, candidate-scan branches, clock syscalls), not from
  algorithmic cleverness — the cheapest, most reliable axis so far.
- Placement (MRU promotion) and per-entry footprint (slim `Item`) are
  orthogonal: each pays off alone and they stack cleanly when merged,
  with no hit-rate trade.
- Trimming metadata the active policy never reads is near-free, improving
  both size (via collapsed alignment padding) and latency.
- A *gated* promotion that steers hot keys toward the MRU head lifts
  retention; widening the gate modestly to a proportional window adds a
  small further hit-rate gain while also shortening buckets.

## Recurring failures

- Ungated / unconditional promotion does not pay off: promoting every
  re-access floods the probing tier and evicts otherwise-resident keys,
  costing hit-rate. Promotion volume must be filtered, not maximized.

## Open questions

- Gate *shape* rather than width: is there a better fraction or a
  recency-weighted (non-positional) gate that reaches warm keys the
  window still misses, or is the trace's protectable mass near-exhausted?
- With per-entry metadata at its floor, where does the next size win come
  from — bucket/segment/container structure rather than `Item` layout?
- Do the footprint floor and the wider promotion gate compound cleanly,
  or does the wider gate's shorter-bucket effect change placement?

## Per-policy notes

- ClassicLRU (active eval policy): purely positional — ignores recency
  and frequency metadata, so any such fields on `Item` are dead weight.
  Successive trimming has driven `Item` to its byte floor; the metadata
  size axis is effectively exhausted under this policy.
