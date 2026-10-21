# Variant v001 Insight

## Helped

(No measured dimension moved. The fitness vector reports only `hit-rate`. The change targets `per-op-time`, which was not measured, so the expected structural win is still unconfirmed. It comes from three code changes. `get` and the `put` residency check now go through `index: HashMap<K, usize>` instead of scanning `small` and then `main` (~2048 entries). The ghost check is now `ghost_set.remove`, replacing `ghost.iter().any` + `retain` over up to ~1946 keys on every miss. `evict_main` rotates `usize` slot ids rather than moving whole `Entry<K,V>` values, which matters more under `MAX_FREQ = 7` because sweeps are longer.)

## Hurt

(No measured dimension regressed. `hit-rate` held exactly, which is what the hypothesis predicted. Two costs are expected but not measured. First, metadata size grows: every resident key is now stored twice (slab and `index`), every live ghost key twice (`ghost` and `ghost_set`), and the ghost deque can hold up to `2*main_size + 16` stale `(K, u64)` elements before compaction. Second, `ghost_insert` has an occasional O(main_size) `retain` spike when it compacts, while v003's per-miss scan cost was steadier.)

## Takeaway

The slab/index/lazy-ghost structure can be swapped under v003's policy with no behavioural change. The unchanged `hit-rate` agrees with the new `matches_v003_reference_on_pseudo_random_streams` test, which checks that small, main and ghost order and per-entry freq match the linear v003 code step by step. So the representation and the policy are now cleanly separated. This variant is a neutral base for later policy probes, such as raising `MAX_FREQ` again or re-tuning the `capacity / 20` split. But its reason for existing, lower `per-op-time` (especially the longer `evict_main` sweeps that `MAX_FREQ = 7` causes), can't be confirmed until the evaluator measures `per-op-time` and metadata size. Until then, this variant only shows that the structure doesn't change behaviour; it doesn't show that it's faster.
