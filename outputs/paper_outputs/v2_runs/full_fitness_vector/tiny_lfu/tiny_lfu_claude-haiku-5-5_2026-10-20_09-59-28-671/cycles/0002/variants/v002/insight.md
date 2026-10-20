# Variant v002 Insight

## Helped

- metadata-size: Packing the protected flag into an existing `Node` field and cutting the resident-list anchors from four to two shrank both per-entry and fixed cache metadata relative to v003, which had added a `bool` to every node and extra anchors to the cache.

## Hurt

- per-op-time: Get and put tail latencies rose relative to v003 even though the hypothesis did not target timing; the extra flag accessors on `touch()`, `unlink()`, and the promotion path plausibly add work per operation. This is a single run, so the size of the regression is uncertain.

## Takeaway

Keeping the probation/protected split while dropping the dedicated `bool` and most anchors recovers the metadata cost without changing hit-rate, which stayed level with v003, so the hit-rate gain in v003 did not depend on the extra bookkeeping. The node shrank further than H02 predicted, which suggests the layout change differs from the one described in the hypothesis; verify the packed field and the bit layout before relying on the metadata figure.
