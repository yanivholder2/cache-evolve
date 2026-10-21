# Variant v001 Insight

## Helped

(No measured dimension improved. The fitness vector reports only `hit-rate`. The memory win the hypothesis claimed, 8-byte fingerprints in a `Vec<u64>` ring instead of cloned keys in `ghost_q` plus `ghost_set`, was not measured because `metadata-size` and `per-op-time` are still absent.)

## Hurt

- hit-rate: Raising `ghost_cap` from `capacity - small_target` to `2 * capacity` sends many more workload-a misses straight into `main` with `freq = 0`, skipping the `small` probation filter. Keys from loops longer than the cache are among the extra ghost hits, and these keys still cannot stay resident until their next reference. Each one fills a `main` slot and pushes out an entry with a shorter reuse distance before its own next reference arrives. A smaller second effect: `ghost_take` leaves a stale ring slot behind, so when that slot is later overwritten, `ghost_dec` can remove a newer push of the same fingerprint early. This makes the ghost's effective window slightly shorter and less exact than `2 * capacity`.

## Takeaway

On workload-a, a longer ghost window does not help with loops longer than the cache. A ghost hit only proves that the key comes back eventually, not that it comes back within `main`'s residency window. Admitting those keys straight into `main` churns entries that would otherwise have hit. The fingerprint-ring mechanism may still be worth keeping for its memory cost, so a follow-up should separate the two parts of this change: keep the fingerprint ring at the parent's `capacity - small_target` size, then measure `metadata-size` and `per-op-time` to check whether the compact ghost pays off without the larger window.
