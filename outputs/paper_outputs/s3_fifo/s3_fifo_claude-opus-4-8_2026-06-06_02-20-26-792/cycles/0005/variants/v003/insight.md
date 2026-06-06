# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Switching the ghost-readmission `put()` and small→main promotion sites from `push_front` to `push_back` drops re-admitted and promoted keys at the LRU tail, so the very next `evict_main` scan reaches them before they get a reuse hit — they are dropped pre-reuse and the parent's hits on those keys are lost.

## Takeaway

A ghost re-admission and a multi-hit small→main promotion are already strong enough signals to deserve initial front protection; treating them as "unproven" and demoting them to the eviction-facing tail is too harsh on this LIRS trace. Reserving the MRU front exclusively for `get()`-reuse evicts these freshly-promoted keys before they can demonstrate reuse, so the entry position the parent gave them was load-bearing, not waste.
