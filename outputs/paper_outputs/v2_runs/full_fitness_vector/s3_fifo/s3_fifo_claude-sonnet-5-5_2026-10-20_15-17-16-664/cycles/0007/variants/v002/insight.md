# Variant v002 Insight

## Helped

- hit-rate: The "recent-main" filter gives freq-0 small-queue victims that were recently evicted from main one reinsertion. The effect is real but negligible, so only a very small number of eviction decisions changed on this trace.

## Hurt

- metadata-size: The headline per-entry size is unchanged. The whole-cache struct grew because of the added ring, its membership flags and the bookkeeping for the filter. As the parent's insight noted, the summary does not count this footprint.
- per-op-time: Median and tail `get` and `put` got slightly slower. The small-eviction path now checks filter membership and can reinsert an entry, which adds work and lengthens the put tail.

## Takeaway

Limiting the filter to a one-retry second chance in small-queue eviction, with main admission and promotion untouched, was almost neutral for hit-rate. Few small-queue victims were both freq-0 and recently evicted from main, so the extra state and eviction-path work bought almost nothing. Later variants should not pursue this tie-break unless a trace shows many such victims. (I could not read the overlay source in this session, so this rests on the hypothesis text and the fitness deltas.)
