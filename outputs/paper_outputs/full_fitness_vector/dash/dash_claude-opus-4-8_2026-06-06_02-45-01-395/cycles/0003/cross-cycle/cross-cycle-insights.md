# Cross-Cycle Insights

## Recurring wins

- Removing hidden hot-path cost reliably trims latency: O(n) front-shifts,
  candidate-scan branches, and `Instant::now()` syscalls were all dead weight
  the exercised policy never needed.
- Steering *where* a hot key lands (MRU placement) is the single most durable
  hit-rate lever, and it stacks cleanly with footprint slimming — placement and
  per-entry size proved genuinely orthogonal, never trading against each other.
- Trimming `Item` metadata the active positional policy never reads shrank
  entries and sped ops for free, collapsing alignment padding at no hit-rate
  cost — until the field count bottomed out.

## Recurring failures

- Promoting *more* keys without a gate floods the probing tier and evicts
  resident keys; raw promotion volume is a hit-rate loss, not a gain.
- Once MRU placement is in play, further promotion-gate tuning (wider windows,
  positional→frequency switches) yields only noise-level hit-rate gains —
  gate shape and MRU placement are substitutes, not additive.
- Chasing hit-rate by touching the hot `get` path (promotion cascades, per-hit
  counter read-modify-writes) costs real latency for no measurable return.
- Micro-latency reorders that remove a single scan per hit sit below the
  harness noise floor — they don't register and can even move the wrong way.

## Open questions

- With placement and gate shape both exhausted on this trace, is any hit-rate
  lever left that isn't noise-limited, or is hit-rate fully tapped?
- The next size win must come from container/segment structure rather than
  per-entry layout — what structural change is large enough to clear the
  measurement noise floor?
- What change granularity *is* resolvable by this harness, given single-scan
  per-hit savings are not?

## Per-policy notes

- `ClassicLRU` (active eval policy): purely positional — ignores recency
  timestamps and frequency/LFU counters entirely. This is why metadata
  slimming was near-free, why MRU placement dominates hit-rate, and why
  frequency-driven gates add nothing over positional MRU. The protectable
  warm mass on this trace sits near the MRU head and is now near-exhausted.
