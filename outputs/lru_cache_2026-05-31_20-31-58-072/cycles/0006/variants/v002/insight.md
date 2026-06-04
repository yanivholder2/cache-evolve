# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Force-demoting a Hot slot on every Cold→Hot promotion (and every ghost-hit Hot insert) past `capacity/2` evicted second-touched keys faster than the workload's reuse distance, so reuses that the parent's uncapped Hot pool was catching now miss; the position-opposite-the-hand cursor in `force_demote_one_hot` picks whatever Hot slot it finds first rather than the truly oldest, costing some genuinely Hot residents.

## Takeaway

On lirs:multi1@512 the unbounded Hot pool was load-bearing, not bloat: the second-touch residents the parent let accumulate were being re-referenced often enough to outweigh the Cold-side recency window the cap was meant to protect. A hard `hot_cap = capacity/2` is too aggressive for this trace — Hot demotions need to be driven by ref-bit evidence (the existing pass-2 sweep) rather than by a fixed quota, or the cap needs to live high enough (e.g. 3·capacity/4) that only genuinely runaway Hot growth triggers it.
