# Variant v002 Insight

## Helped

- hit-rate: Scanning the LRU quarter of the protected segment (roughly 410 nodes at capacity=2048) rather than a fixed tiny window gave the frequency selector enough candidates to consistently find genuinely infrequent entries for demotion, while halving rather than zeroing counters on demotion preserved accumulated signal so nodes with real access history remained distinguishable from newly promoted ones on subsequent eviction decisions.

## Hurt

## Takeaway

The improvement confirms that cycle/0002/v002's failure was a scan-window problem rather than evidence that frequency is uninformative inside the protected segment: a window proportional to the segment size (25%) creates meaningful selection pressure, whereas a fixed 4-node window over ~1600 slots is functionally indistinguishable from strict LRU. Counter halving on demotion is a necessary complement — without it, every demotion cycle resets the playing field and the frequency signal built up during a node's protected residence is discarded at the exact moment it would be most useful for re-ranking on the next overflow.
