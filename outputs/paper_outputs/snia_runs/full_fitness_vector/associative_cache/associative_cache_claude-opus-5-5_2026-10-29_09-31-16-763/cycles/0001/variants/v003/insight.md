# Variant v003 Insight

## Helped

- metadata-size: Did not move, as intended. Reusing the existing `Item.lfu_counter` field as ClassicLRU's 2-bit hit counter adds frequency awareness without growing `Item<u64,u64>` or the cache struct.

## Hurt

- per-op-time: The tail latencies on put, and put p50, went up, while get p50 stayed flat. This fits the cost being on the eviction path: every eviction into a full bucket now scans an 8-slot window for the minimum counter, runs a second pass to age the other window items, and compares a `name() == "ClassicLRU"` string. The only cost added to get is one saturating increment before the move-to-back.

## Takeaway

hit-rate came out exactly the same as the baseline on every web_0/hm_0 trace and size, down to the last digit. A change that picks a different victim whenever a hit item sits at the LRU front should not do that. So the frequency-aware victim window was most likely never used in the hit-rate run, rather than tried and found useless. For example, the hit-rate harness may not run the overlaid `AssociativeCacheBucket::evict_item` or the ClassicLRU policy. This needs checking before the idea is judged. Even if it is wired in, the design protects reused items only weakly. A hit moves the item to the back, outside the window, so it can only use its counter after drifting back to the tail, and with a cap of 3 and one decrement per eviction, a protected item can survive only a few evictions there. The latency cost, by contrast, is paid on every eviction.
