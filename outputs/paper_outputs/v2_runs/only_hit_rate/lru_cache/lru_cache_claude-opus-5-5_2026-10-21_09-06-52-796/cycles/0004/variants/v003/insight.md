# Variant v003 Insight

## Helped

- hit-rate: Once the cache is full, a ghost-hit key now goes into `main` only if its count-min sketch estimate is strictly higher than the estimate for `nodes[main.front()]`. A loop key that comes back just once inside the `capacity / 2` ghost window now falls through to `small` instead of displacing `main` content. This gave a small gain, but only a marginal one. The gate filters re-admissions that v001's tighter ghost window had already mostly cleaned up.

## Hurt

(No measured dimension regressed. The fitness vector reports only `hit-rate`. Two costs went unmeasured. First, the sketch hashes every key twice with SipHash (`DefaultHasher`): once on each `get` and on each new-key `put`, and again for the two `estimate` calls on a ghost hit. Second, it adds a `4 * next_pow2(capacity)` byte table, which is 8 KiB at size 2048, plus a full-table halving pass every `10 * capacity` accesses. So `per-op-time` and `metadata-size` are likely worse, but that is unverified.)

## Takeaway

On workload-a, after v001's ghost shrink, a frequency gate on ghost-hit re-admission adds almost nothing. The quality of re-admission decisions is not where most of the remaining gap to LIRS lies. There are three structural reasons the gate has little effect:

- **It compares against the wrong entry.** It checks against `main.front()`, but `evict_one` usually takes its victim from `small`, because `small.len() >= small_target`. When it does use `main`, it may rotate the front entry instead of evicting it. So the comparison rarely matches the entry that actually gets displaced.
- **Rejected keys get a second chance anyway.** A key that loses the gate still enters `small`. One hit while it is on probation promotes it to `main`, so rejection mostly delays these keys rather than filtering them out.
- **Ties count as rejections.** The `>` test is strict, the width-2048 sketch is noisy, and counters are halved every 20480 accesses. Under these conditions ties at low counts are likely common, and every tie sends the key to `small`.

If the sketch is kept, a better use would be TinyLFU-style admission against the actual eviction victim (candidate vs. the key `evict_one` would remove), not a gate on the ghost path alone.
