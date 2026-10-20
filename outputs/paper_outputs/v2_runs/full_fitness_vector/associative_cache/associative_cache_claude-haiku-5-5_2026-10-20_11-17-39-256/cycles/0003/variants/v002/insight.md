# Variant v002 Insight

## Helped

- per-op-time: The victim search and the counter halving now share one pass over the bucket on put, instead of a separate minimum search before removal, so put got faster while get was untouched.

## Hurt

- hit-rate: The halving runs on every put into a full bucket, so a slot's frequency counter (capped at 3) is reduced to zero after about two evictions in its bucket. That erases the protection that frequent hits were supposed to provide, and it makes the decay track the miss rate rather than elapsed time.

## Takeaway

The hypothesis assumed v003 never decays its counters, but the parent already halves every counter every 256 insertions and caps them at 3, so stale history was already bounded. This variant stacks a second, miss-driven decay on top of that, and the hit-rate loss shows that the extra decay is too aggressive at this counter width. Any follow-up on aging should drop the per-eviction halving and test whether the existing 256-insertion period or a wider saturation range is the right lever.
