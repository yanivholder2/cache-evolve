# Cycle 0002 Insight

## Cohort outcome

All three variants built on cycle-0001 v001 (MRU-end stash demotion), and each added a different stash lever. v001 added a fixed two-hit exit gate, v002 added a TinyLFU-style frequency comparison against the target's LRU victim, and v003 inserted fresh puts at the stash LRU end with a BIP escape. v001 won with small gains at the small size and on `hm_0` at the large size. It regressed on `web_0` at the large size, the same weak spot as cycle-0001's gate.

## Patterns

- All three levers act on the same small population: items sitting in tiny, bucket-local LRU stash slots. That makes them interact rather than stack. v001's counter reset on demotion penalizes exactly the demoted items that `Back` placement favors. v003's `Front` insertion behaves like "evict on next collision", so new keys lose their probation hit.
- `web_0` at the large size stayed the hardest cell. No variant recovered it, and every extra filter (gate, comparison, LRU-end insertion) cost hit-rate there. When target buckets have spare room, first-hit promotion is already close to optimal.
- Gains appeared mainly where reuse comes soon or contention is high: small size for the gate, `hm_0` at the large size for v002 and v003. Results split across cells far more often than they moved together.
- Frequency signals are weak in this design. Promotions land at the target's eviction end, and counters are halved or reset on moves, so counters stay near zero. v002's comparison therefore ended up acting like plain first-hit promotion.
- Per-op-time and metadata-size were again not reported. All cost claims are still unverified.

## Open questions

- Does the gate help without regression if it applies only to fresh puts, so demoted items keep their count or promote on their first stash hit?
- Would a frequency comparison actually bite if it compared against the bucket's lowest counter, or if promotions didn't insert at the LRU slot?
- Can fresh-key insertion be softened (one slot above LRU, per-bucket bimodal counter) so it keeps the `hm_0` large-size gain without starving new keys?

## Continuation

Build on this cycle's v001. The top candidate is to restrict the two-hit gate to fresh puts and stop resetting counters on demotion, since that targets the `web_0` large-size regression directly. If frequency admission is retried, fix the victim choice (lowest counter, not LRU index 0) first. Drop hard LRU-end insertion of fresh puts. Make sure per-op-time and metadata-size are measured before any cost claim is made.
