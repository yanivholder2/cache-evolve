# Variant v002 Insight

## Helped

_(none)_

## Hurt

_(none)_

## Takeaway

Adding a secondary 2-position scan above the primary 8-slot cold window — guarded so it only fires when every item in the primary window is warm — produced no measurable change on any fitness dimension. This indicates that on this workload, the "all-warm primary window" scenario either occurs too rarely to affect aggregate hit-rate, or the two additional positions above the window are themselves consistently warm when that scenario does fire, so the secondary scan almost never finds a cold victim and the fallback to `len - 1` is unchanged. The conditional nature of the expansion is its own limitation: it targets a narrow eviction scenario and adds zero cost when that scenario is absent, but it also delivers zero benefit, suggesting the hot set does not routinely overflow all 8 cold-window slots simultaneously under this access pattern.
