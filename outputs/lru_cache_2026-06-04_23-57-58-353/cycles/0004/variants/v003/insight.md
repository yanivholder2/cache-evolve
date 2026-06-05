# Variant v003 Insight

## Helped

- hit-rate: Scanning the LRU half of probationary for the minimum-frequency victim instead of blindly evicting the head allowed cold newcomers (freq=0) to be chosen over entries recently demoted from protected (freq>0 after counter halving), keeping recently-warm keys alive long enough for a second promotion opportunity.

## Hurt

## Takeaway

The frequency-aware demotion introduced in v002 preserved signal when entries left the protected segment, but that signal was immediately discarded by strict LRU at the probationary eviction site — a demoted entry with a halved but non-zero freq could still be the probationary head and get evicted on the very next insertion. Extending frequency awareness to the probationary eviction path with a half-list scan closes this gap: the same freq counter that survived demotion now actively shields the entry from cold-traffic pressure. The improvement confirms that the counter-halving strategy is only fully effective when both eviction sites respect it.
