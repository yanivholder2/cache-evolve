# Cross-Cycle Insights

## Recurring wins

- Cheap, near-pure hot-path changes that improve one dimension without
  trading against another have won so far — the inline integer hasher
  improved per-op-time and even threw off a free metadata side-win.

## Recurring failures

- Single-axis levers that move their target dimension but pay for it with
  added per-op latency have not paid off — extra promote/demote branching
  and per-access index-width casts both regressed the contested per-op-time
  axis.
- Widening node representation or sprinkling per-access conversions did not
  earn back its cost in locality.

## Open questions

- Can the SLRU hit-rate lever be reintroduced without its latency cost? It
  targets the LIRS workload's real weakness but currently trades against
  per-op-time.
- Is there a cast-free narrow-link encoding that lands the metadata win
  without per-access width casts?
- Does the hasher's get tail-latency regression need attention, or is the
  median win worth it as-is?

## Per-policy notes

- LRU baseline: per-op-time is the most contested axis; hot-path purity
  decides cohorts. Hit-rate gains have so far come from a segmented-LRU
  variant aimed at the LIRS workload, but at a latency cost not yet
  resolved.
