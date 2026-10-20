# Variant v002 Insight

## Helped

## Hurt

## Takeaway

Narrowing the intrusive list links and the map's stored index from `usize` to `u32` is a layout-only change: eviction, admission, and the sketch are untouched, so hit-rate is expected to stay identical, and it does. Its only intended payoff is in metadata-size, but this variant's fitness vector reports only hit-rate, so the per-entry node shrink is unmeasured. The new `capacity > u32::MAX` guard in `try_new` is a real behavioural cost: such caches now fail construction instead of being built.
