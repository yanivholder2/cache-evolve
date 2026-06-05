# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Re-admitting ghost keys at the protected-MRU end injects keys with only a single post-eviction reuse straight into the segment that already holds ~95% of capacity, and each such admission can trip `enforce_protected_budget` into demoting a genuinely tenured protected-LRU key down to the 103-slot probationary segment, where it is churned out before it can be reused — so the change displaces proven winners with one-touch candidates and loses hits net.

## Takeaway

On the 95/5 incumbent the protected segment *is* essentially the whole cache, so warm-end ghost re-admission is not free protection — it dilutes the hot set and forces demotions through the already-binding probationary floor. The untested "placement" variable the hypothesis isolated turns out to hurt at this ratio: fast post-eviction reuse is too weak a signal to earn maximum protection when protected room is the scarce resource being spent.
