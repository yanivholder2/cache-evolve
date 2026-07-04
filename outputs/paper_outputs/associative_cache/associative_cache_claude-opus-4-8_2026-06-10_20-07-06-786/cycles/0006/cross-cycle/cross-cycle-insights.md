## Recurring wins

- Demanding *proof of reuse* before granting protection is the central
  winning idea: frequency-first victim selection (C1), explicit segmentation
  with promote-on-first-hit (C2), and FIFO probation that gives a fresh key a
  full lap to earn promotion (C3) each lifted hit-rate.
- Durable protection pays off — once a key is proven hot it should stay
  protected. "Once-hot immortality" and promote-straight-to-MRU placement
  have repeatedly outperformed schemes that re-test or re-rank earned keys.
- When behavioral logic saturated, *geometry* became the live lever: a single
  bucket-count halving (wider associativity, capacity conserved) gave the
  first hit-rate gain in several cycles (C6).
- Hit-rate gains have come from orthogonal levers (victim rule, admission,
  probation order, associativity) — attacking the resident set from outside
  the saturated axis tends to be where the next gain hides.

## Recurring failures

- Any change that *erodes earned protection* regresses. This single boundary
  explains nearly every loss across cycles: counter aging (C2), front/
  probationary admission composed with frequency (C2), two-hit promotion
  thresholds (C4, C6), protected-segment caps with demotion (C3 inert, C4
  regress), CLOCK/recency approximation (C4), LFU-min protected-eviction
  fallback (C5), ghost-ring accelerated re-entry (C5), and re-seating
  promotions below MRU (C6).
- "Recently evicted" and "high banked count" are not reliable proxies for
  "hot" on this trace — re-entry and frequency re-signaling both fail to
  separate hot from rising keys better than plain protected-LRU recency.
- Geometry coarsening beyond the sweet spot gives diminishing returns:
  early bucket widening was collision-bound (C1) and the single halving is
  already described as small/near-optimal (C6).

## Open questions

- Is hit-rate now at its ceiling for behavioral logic, leaving geometry and
  segment sizing as the only live axes?
- Is one bucket-count halving the optimum, or does a finer split — or a
  geometry that interacts with segment sizing — extend the gain?
- Does *looser/larger* protected sizing (enlarging, not tightening, protection)
  move the needle? It remains the main untested hit-rate direction.
- Cost-side wins (reorder-free tiebreak, no-op reorder elision, hot-path fast
  paths) are real but invisible under hit-rate-only fitness — is a per-op-time
  axis warranted before spending more slots on them?

## Per-policy notes

- Pure LRU: the displaced baseline; loses to any scheme that shields the
  reusable working set from one-hit-wonders and scan traffic.
- LFU / frequency: a durable, non-aged frequency signal won as a victim rule
  (C1), but frequency re-signaling for protected eviction and counter aging
  both regress — frequency helps as long-memory protection, not as a re-tester.
- Segmented-LRU: the settled behavioral core. Single-hit promotion, promote-
  to-MRU placement, and a load-bearing (uncapped) protected segment are all
  confirmed; promotion-tightening and protected caps repeatedly fail.
- FIFO probation: oldest-first eviction of zero-count entries is the strongest
  probation order — it grants rising keys a promotion window without changing
  admission or metadata.
- Geometry / associativity: the currently most promising axis; a mild
  associativity-starvation responded to one bucket-count halving, but the
  regime appears near its associativity sweet spot.
