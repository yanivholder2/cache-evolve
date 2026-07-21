# Variant v002 Insight

## Helped

*(no dimension moved favourably)*

## Hurt

- hit-rate: The vacancy guard (`!target.is_full()`) silences probing re-entry precisely when Zipfian hot items are most likely to be resident in probing — high cache occupancy — so the mechanism fires too rarely to offset the small overhead of incrementing `lfu_counter` on every probing hit.

## Takeaway

Grafting vacancy-gated probing re-entry onto v003's clock-eviction base produced no measurable hit-rate gain: the two-hit counter threshold and the full-target guard together form a narrow firing window that a Zipfian workload largely keeps closed, because the hottest items promoted to probing tend to have a full target behind them. The mechanism is structurally sound but its effectiveness depends on how often the target tier has headroom; on a saturated cache it contributes nothing, revealing that the passive probing path in v003 already handles warm probing residents as well as this active re-entry scheme.
