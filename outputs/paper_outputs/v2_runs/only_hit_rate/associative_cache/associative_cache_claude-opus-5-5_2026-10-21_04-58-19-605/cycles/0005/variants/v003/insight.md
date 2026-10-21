# Variant v003 Insight

## Helped

(No dimension improved. `hit-rate` on workload-a at 2048 is identical to the parent's figure, so it did not rise above that figure. Under H03's own refutation clause, the hypothesis is refuted.)

## Hurt

(No measured dimension regressed. Hit rate did not move, which most likely means the lenient branch (`grace >= GRACE_ON`) almost never fired. The bucket kept running the parent's strict path. That costs a `u8` plus an `Option<usize>` of state per bucket and an added compare on the full-bucket miss path, with nothing gained. Neither cost is in the fitness vector.)

## Takeaway

The adaptive duel could not leave strict mode because its controller feeds on its own outcome. In strict mode the hand rests on a zero-count newcomer, so on the next miss that newcomer is evicted at once. Each such eviction counts as a "never-hit" negative, which decrements `grace`. Positives arrive at most once per ghost-ring match, and the ring holds only 4 entries. The score starts at 0, so it would need a net of 8 ghost hits to cross `GRACE_ON`, and the strict filter keeps pushing it back to 0.

The negative signal is also biased. A ghost-admitted newcomer starts at counter 1 and can be swept down to 0 without ever being hit, yet its later eviction still counts as a never-hit eviction. The rationale is not refuted outright, because the lenient filter was effectively never tested. Before concluding that "strict plus ghost admission is already the right shape", a retry should change three things:
- Measure the negative signal under a neutral policy, or only on sampled leader buckets, as in DIP set-dueling.
- Count only newcomers admitted at counter 0 as negatives.
- Seed `grace` at or near `GRACE_ON`.
