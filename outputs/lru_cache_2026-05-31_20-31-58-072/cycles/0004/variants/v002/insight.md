# Variant v002 Insight

## Helped

## Hurt

## Takeaway

Replacing pass 1's first-Cold-ref-clear pick with a full-sweep
least-recently-touched choice over the same Cold ref-clear pool left
hit-rate on lirs:multi1@512 unchanged from v001. On this trace the two
selection rules pick effectively interchangeable victims: by the time the
CLOCK hand encounters a Cold ref-clear slot, the Cold pool has already
been aged uniformly enough that hand-position is a good proxy for
last-touch order, and the keys distinguished by `last_touch` are not the
ones that get re-referenced before they would have been evicted anyway.
Adding recency *within* the Cold pool is not where the residual gains
live on lirs:multi1 — the next lever has to change which slots become
Cold, or how the Hot/Cold boundary is sized, rather than re-ranking
victims inside an already-doomed Cold set.
