# Cross-Cycle Insights

## Recurring wins

- Latency wins have come from removing hidden hot-path cost (O(n) front-shifts,
  candidate-scan branches, per-access `Instant::now()` syscalls) rather than
  algorithmic cleverness.
- Steering *where* a promoted hot key lands (e.g. MRU placement) lifts retention
  without churn — a safe, repeatable direction.
- Trimming `Item` metadata that the active policy never reads is a near-free
  win on both size and latency.

## Recurring failures

- Promoting *more* keys unconditionally floods the probing tier and evicts
  otherwise-resident keys; hit-rate behaves as a balance, not a free lunch. The
  hottest-only promotion gate is load-bearing.

## Open questions

- Does gated promotion combined with MRU placement compound the hit-rate gain,
  or do the two levers cancel?
- Is there a middle promotion gate between "single hottest" and "every
  re-access" that widens the protected set without churning the probing tier?
- Under a positional-only eval policy, how much remaining `Item` footprint is
  dead weight?

## Per-policy notes

- `ClassicLRU` is purely positional: it ignores recency timestamps and LFU
  counters, so any such metadata on `Item` is pure overhead on size and latency
  under this policy.
