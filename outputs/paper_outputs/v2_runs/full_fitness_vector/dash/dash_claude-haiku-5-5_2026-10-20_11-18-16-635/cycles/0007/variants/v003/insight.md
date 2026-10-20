# Variant v003 Insight

## Helped

- hit-rate: Porting the saturating access counter and the full-scan lowest-counter victim choice onto the packed `Item<u64,u64>` base moved hit-rate up relative to the parent, but only a fraction of the gain the hypothesis targeted, so the counter-based victim choice carries over to the packed layout only partially here.
- per-op-time: Get and put latencies were slightly lower than the parent. The only get-path change is one saturating increment on a target hit, which should add cost rather than remove it, so this improvement is not explained by the change and is more likely run-to-run noise than a real effect.

## Hurt

## Takeaway

The layout and replacement changes combined without costing metadata: the packed record stayed at its parent size, which supports the hypothesis's claim that eviction policy and record packing are separable. The gain from the counter-based victim choice was modest, well short of the target the hypothesis set, so the claim that the two incumbents' benefits simply add is not supported by this run. The result rests on a single trace and the overlay source could not be inspected in this session, so whether the counter sits in spare bits or the packed `u8` slot, and whether the second-trace check passes, remain unconfirmed.
