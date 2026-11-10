# Variant v003 Insight

## Helped

- hit-rate (web_0 @ 16K only): the only point that moved up, and only slightly. The likely cause is ghost hits growing the small window enough to catch some reuse just past the fixed 25% window on the trace and size where that window was most cramped.
- per-op-time: improved on both `get` and `put`, but the diff can't explain it. The change only adds work: `while` eviction loops, an `enforce_total` pass after every put, and resize bookkeeping. The lookup path (linear scans over small+main and ghost) is the same as in baseline. Treat this as a difference in the measurement environment, not as a property of adaptive sizing, and re-measure before crediting it.

## Hurt

- hit-rate: the summary and three of the four trace/size points fell, hm_0 @ 131K the most. The two signals are probably not balanced. Any second-chance reinsertion during an `evict_main` call shrinks small. In a main that holds a hot working set, that happens on almost every main eviction. Ghost hits are the only thing that grows small, and they are rarer. Worse, each ghost hit both grows small (shrinking `main_size`) and pushes into main, which forces `evict_main`, which in turn shrinks small. So the controller keeps undoing its own moves instead of settling on a split.
- hit-rate (131K): with `delta = capacity/1000`, each step at the large size moves over a hundred slots at once. The ghost queue also stays sized to the *initial* main, so it no longer matches main's actual size once the split drifts. This likely explains why the 131K points regressed more than the 16K points.
- metadata-size: the per-entry size is unchanged, so the summary didn't move. The `S3Fifo` struct itself grew by the six new controller fields (`capacity`, `min_small`, `max_small`, `delta`, `ghost_size`, `adaptive`). This is a fixed per-cache cost, small but real.

## Takeaway

Adding an ARC-style adaptive split to S3-FIFO doesn't help if the two signals fire at very different rates. A shrink on every reinsertion in `evict_main` outvotes the grow on ghost hits. The ghost-hit path also triggers the opposite signal itself, because re-admitting into a now-smaller main forces a main eviction. A follow-up should balance the signals (e.g. normalize by queue sizes as ARC does, or shrink only per *evicted* main entry). It should also size the ghost queue to track `main_size`, and use a smaller step at large capacities. Until then, the adaptive split doesn't beat the fixed 25% split.
