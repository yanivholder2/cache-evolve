## Recurring wins

- Structural changes win; small tweaks do not. Wider buckets (geometry) gave a small gain. A probationary segment (admission control) gave a clearly larger one.
- Separating one-touch entries from re-referenced ones is the most productive lever so far. Policy, not hash balance, appears to limit hit rate on this trace.

## Recurring failures

- Weakening hit-path promotion in short buckets (transposition, midpoint, half-promotion) lowered hit rate. Full move-to-back recency is hard to beat at small associativity.
- Small-window frequency tie-breaks at the LRU end changed nothing measurable.
- Multi-bucket or two-choice placement fails the slot-legality invariant, even on the baseline. Any key placed outside its hashed bucket is rejected as the gate is written.
- Per-op-time and metadata-size have been missing from the fitness vector in every cycle. Speed and size claims in all hypotheses remain untested, and hit-rate wins are unproven as net wins.

## Open questions

- Fix measurement first: can per-op-time and metadata-size get into the fitness vector? Do the wider-bucket and probation gains survive their added scan and bookkeeping costs?
- What probation size is best (smaller or larger than the current share)?
- Does a real probation-to-protected frequency test, or ghost-history information, add anything on top of the probationary segment?
- Should the slot-legality invariant be deliberately relaxed to allow placement experiments, or should they be dropped?

## Per-policy notes

- Plain LRU (move-to-back) buckets: strong baseline. Promotion-weakening variants lose to it.
- Probationary segment (2Q-like, separate region): best result so far. It builds on the wide-bucket geometry, and the tuning space is still open.
- Frequency-aware victim choice: no effect at a small window. A wider window or a promotion test is untried.
- Two-choice or multi-bucket placement: blocked by the invariant, with no performance evidence either way.
