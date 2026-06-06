# Variant v003 Insight

## Helped

## Hurt

- hit-rate: The periodic `tick`-driven sweep that halves every main entry's `freq` once per `main_size` puts strips the second-chance counter from genuinely hot keys faster than reaccesses can rebuild it, so `evict_main` reaches and discards keys that the parent's lazy-only decay would have kept resident on this LIRS trace.

## Takeaway

Adding a time-based global aging sweep on top of the move-to-front main is too blunt a lever: uniform halving ages active and stale residents alike, and because `freq` only rebuilds one bump per reaccess, the sweep evicts still-hot keys before they recover. Eviction-time decay during the `evict_main` rotation was already aging the cold tail precisely; a periodic global halving over-decays and lowers hit-rate rather than raising it.
