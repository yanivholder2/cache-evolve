# Variant v003 Insight

## Helped

- metadata-size: The added `evict_count: Cell<u32>` sits on `AssociativeCacheBucket`, so it stays invisible to the `Item<u64,u64>` and `AssociativeCache<u64,u64>` measurements that fitness samples — the cadence machinery costs nothing in the reported metadata vector.

## Hurt

- hit-rate: Every `max_size`-th eviction unconditionally drops the back (newest) slot, overriding v003's `lfu_counter == 0` filter and discarding items that have just been inserted from the access path — on this LIRS trace, newly arrived slots are often genuine working-set items, so the periodic sacrifice ejects exactly the keys the parent's filter was already protecting from death by churn.

## Takeaway

Layering an unconditional cadence on top of v003's signal-driven filter actively destroys information: v003's filter already distinguishes scan slots (`lfu_counter == 0`) from earned slots, but the 1-in-K newest-slot kill fires regardless of that bit and is just as likely to evict a hot newcomer as a scan key. In Dash bucket-movement, "newest slot" is not a reliable proxy for "scan victim" — once the parent has a working signal for one-shot detection, additional eviction overrides that ignore that signal subtract from, rather than add to, the protection it provides.
