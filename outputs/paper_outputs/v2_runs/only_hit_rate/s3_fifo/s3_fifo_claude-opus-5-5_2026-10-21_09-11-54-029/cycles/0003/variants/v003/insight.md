# Variant v003 Insight

## Helped

- hit-rate: Raising `MAX_FREQ` from 3 to 7 lets `evict_main` tell keys with a few reuses apart from keys with many. A heavily reused key now absorbs up to 7 decrement-and-`push_front` laps before it reaches the tail at freq 0. So when a burst of ghost re-admissions forces evictions, the CLOCK-style sweep drops keys reused 3–6 times instead of the hottest ones. A related effect sits outside the hypothesis's "retention only" framing: `get` clamps freq the same way in the small queue. A key that graduates through `evict_small` (`freq > 1`) can therefore arrive in main with more than 3 credits already banked. The admission threshold is unchanged, but the protection a key carries into main on promotion is larger.

## Hurt

(No measured dimension regressed. The fitness vector reports only `hit-rate`. The expected cost is in `per-op-time`, which was not measured: the worst-case rotation bound in `evict_main` grows from `main.len() * 3` to `main.len() * 7`. With main at ~1946 slots, the sweep that finds a freq-0 victim can be much longer when most of main is hot.)

## Takeaway

Retention inside main was untouched until this cycle, and adjusting it alone gives a real but small gain on workload-a. Workload-a has some signal beyond "reused at least 3 times", but a cap of 3 was already capturing most of it. Main has ~1946 slots and ghost-admitted keys enter at freq 0, so cold ghost re-entries usually supply the victims before the extra credit on hot keys matters. That keeps the upside of a larger cap modest. The next step should measure the cost before pushing the cap higher. That means measuring `per-op-time`, and especially the length of `evict_main` sweeps, since the extra credits make each sweep longer.
