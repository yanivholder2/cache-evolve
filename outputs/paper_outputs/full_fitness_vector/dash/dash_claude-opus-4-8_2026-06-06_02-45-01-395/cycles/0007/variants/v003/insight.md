# Variant v003 Insight

## Helped

- hit-rate: Held exactly at the parent value — relocating the heat-clear from per-put to the eviction site preserved every gate read, confirming the change is the promised bit-for-bit-on-decisions refactor and not a behavioral one.
- metadata-size: Unchanged; the clear moved call sites but added no per-item state.

## Hurt

- per-op-time: Both put metrics regressed (p50 and especially the p99 tail), and get drifted up too. The parent's per-put `target_heat.remove` was a *negative* lookup on the freshly-inserted, always-non-resident key — a cheap, predictable miss that short-circuits. Relocating it to the stash-eviction sites turned it into a remove on a *resident*, possibly heat-bearing evicted key, and gated it behind `if let Some(evicted)` at three branch points on the colder eviction path, lengthening the put tail rather than shortening it.

## Takeaway

The per-put probe was not what bounded put latency: it was the cheapest possible form of the operation (a non-resident negative lookup), so removing it from the common path and re-spending it on the eviction path — as a real remove behind new conditionals — made put-p50 and put-p99 worse, not better. This answers the cycle-0006 open question negatively: the put-p99 tail is dominated by the eviction/cascade work, not by the side-table touch, so adding branches there is the wrong place to economize.
