# Variant v002 Insight

## Helped

- metadata-size: The cache struct itself got slightly smaller, plausibly because the sketch no longer depends on state shared with the table hash. The sketch heap allocation and node size are unchanged.

## Hurt

- per-op-time: Giving the sketch its own per-row hashing in `increment` and `estimate` roughly doubled get and put latency at both p50 and p99. This undid almost all of the parent's speedup.
- hit-rate: A tiny drop, which is odd for a change meant to leave policy alone. It is probably because the independent hashes pick different sketch counters and so collide differently.

## Takeaway

The flat table and slab were not the main source of cycle 0009's latency gain. Deriving the sketch row indices from the single table hash was, because it removes the extra hash work on every get and put. The shared hash is worth keeping, and its small hit-rate cost is a separate question from the speed it buys.
