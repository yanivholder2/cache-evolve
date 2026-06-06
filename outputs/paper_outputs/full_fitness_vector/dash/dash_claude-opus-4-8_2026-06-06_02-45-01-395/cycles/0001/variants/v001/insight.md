# Variant v001 Insight

## Helped

- hit-rate: Switching the `get` entry point to `InsertPosition::Back` lands a stash→target promotion at the MRU end, so a hot key reaches the protected probing tier in one extra access instead of re-climbing the whole bucket — fewer hot keys are evicted mid-climb on the skewed LIRS trace.
- per-op-time: Inserting at `Back` is a `Vec::push` rather than the baseline's O(n) `insert(0, …)` at the front, which roughly halved get p50/p99 and also trimmed put latency.

## Hurt

- (none) — metadata-size is untouched since insert position changes neither slot layout nor capacity.

## Takeaway

For this Dash segment, the conservative `Front` promotion penalized hot keys twice: it slowed their tier ascent *and* paid an O(n) front-shift on every hit. Flipping promotion to the MRU end fixes both at once — a single-argument change that buys hit-rate and latency together without disturbing any P1–P6 invariant.
