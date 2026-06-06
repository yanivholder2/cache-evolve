# Variant v001 Insight

## Helped

- hit-rate: The k=8 minimum-frequency scan moved hit rate fractionally upward, confirming that occasionally the LRU head of protected has a lower sketch estimate than items slightly deeper in the list and replacing it yields a marginally better survivor.

## Hurt

## Takeaway

Replacing unconditional LRU demotion from the protected segment with a bounded 8-node frequency scan produces only a negligible hit-rate gain on this Zipfian workload. With a 90% protected segment, the LRU end of the protected list is occupied by items that are already reasonably frequent — the sketch rarely identifies a candidate within 8 steps that is meaningfully colder than the head itself, so the scan seldom overrides the default LRU choice. The protected-segment demotion policy is a weak lever at this segment ratio; the hot-set is already well-retained, and further gains will require addressing admission or frequency-decay rather than which item gets demoted when protected overflows.
