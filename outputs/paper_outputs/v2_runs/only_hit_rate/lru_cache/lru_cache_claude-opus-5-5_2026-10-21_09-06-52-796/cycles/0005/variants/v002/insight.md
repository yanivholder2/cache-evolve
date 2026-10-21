# Variant v002 Insight

## Helped

(No measured dimension improved. The fitness vector reports only `hit-rate`, and it matches the parent exactly. The benefits this change targets are `metadata-size` and `per-op-time`, and neither was measured. In code terms the ghost now holds a `Vec<u64>` of `ghost_cap` entries plus a `HashMap<u64, u32>` instead of a `VecDeque<K>` plus a `HashMap<K, ()>`. Small-queue evictions no longer clone `K` into the ghost twice, and main-queue evictions no longer clone the key just to remove it from `map`. All of that should cut ghost footprint and allocation churn, but none of it is verified.)

## Hurt

(No measured dimension regressed. `hit-rate` held exactly, as expected for a change that only alters the representation. One possible cost is unmeasured: every missing `put` now hashes the key with `DefaultHasher` to get the fingerprint, then hashes that `u64` again inside `ghost_map`. The parent did a single `K` hash in `ghost_set.remove`. Small evictions likewise add a fingerprint hash before the `map.remove`. So for cheap keys like `i64`, `per-op-time` could come out flat or slightly worse rather than better.)

## Takeaway

Swapping cloned ghost keys for 64-bit fingerprints in a ring that keeps FIFO order, at the same `capacity / 4` window, left hit rate unchanged on workload-a at size 2048. Fingerprint collisions at roughly 512 live entries are negligible. The stale-erase fix also did nothing measurable here. A key can only be ghosted twice inside one window if it gets a ghost hit, goes to `main`, is evicted from `main`, comes back through `small` and is evicted again, all within `ghost_cap` small evictions, which almost never happens. Because the change is policy-neutral, it is a safe base for later ghost experiments. Whether it is actually worth it depends on running `metadata-size` and `per-op-time`, and on checking that the extra `DefaultHasher` pass per miss doesn't cancel out the savings from dropping the key clones.
