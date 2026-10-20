# Variant v003 Insight

## Helped

- per-op-time: Replacing the parent's per-row `DefaultHasher` (SipHash, run `depth` times per op) with a single `FastHasher` fold plus double-hashing (`base_hash` once, `position_for` per row) collapses both `get` and `put` p50 to roughly a third of the parent's, and tames the eviction-path `put` p99 — the sample walk's repeated `estimate` calls now share one cheap hash instead of paying SipHash per candidate.

## Hurt

- hit-rate: Deriving all `depth` sketch columns from one 64-bit hash via `(h1 + row*(h2|1))` makes the rows less independent than the parent's fresh-SipHash-per-row scheme, slightly raising counter collisions and nudging a few admission decisions the wrong way.

## Takeaway

Swapping per-row SipHash for one FastHasher fold with double-hashed columns buys a ~2.5x per-op speedup that makes v003's sampled-eviction hit-rate affordable, at the cost of a marginal hit-rate dip from the now-correlated sketch rows. The frequency estimate is robust enough that trading row independence for a single cheap hash is strongly net-positive here.
