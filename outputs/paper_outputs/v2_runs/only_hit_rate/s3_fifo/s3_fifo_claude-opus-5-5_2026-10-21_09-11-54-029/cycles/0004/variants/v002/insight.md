# Variant v002 Insight

## Helped

(No dimension moved meaningfully.)

- hit-rate: Effectively flat. The nominal uptick is too small to separate from noise. Raising `MAX_FREQ` from 7 to 15 lets a saturated main key survive up to 15 decrement-and-`push_front` laps instead of 7. However, `evict_main` almost never needs the 8th–15th credit. Ghost re-admissions enter main at the front with freq 0 and reach the tail as ready-made victims, and keys with only a few banked reuses are drained long before a hot key's credit runs out. So the extra credits rarely change which key gets evicted.

## Hurt

(No measured dimension regressed. The fitness vector reports only `hit-rate`. The cost the hypothesis warned about is still unmeasured: the worst-case rotation bound in `evict_main` doubles to `main.len() * 15`, ~1946 × 15 at size 2048. If most of main is hot, each sweep now does about twice as many `pop_back`/`push_front` laps, for no hit-rate return.)

## Takeaway

The cap curve on workload-a is flat past 7. This answers cycle 0003's open question: the frequency signal saturates at about 7 reuses, and the 3→7 gain was the whole payoff from the cap. Further main-retention work should change how credit is spent or aged, for example decay rules or ghost re-entry priority, instead of raising the cap. Because a larger cap can only lengthen `evict_main` sweeps, the cap should go back to 7 unless `per-op-time` shows the longer sweeps cost nothing.
