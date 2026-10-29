# Variant v002 Insight

## Helped

- hit-rate (128K, `web_0`): This is where v001's two-hit gate did the most damage, and most of the gain comes from here. At this size, target buckets often have a free slot. With `target_has_room`, a reused stash item now promotes on its first hit and stops competing with new `put` arrivals for stash space. This includes items that were demoted and had their `lfu_counter` reset, which no longer need two hits to return to a target that has room.
- hit-rate (128K, `hm_0`): Roughly flat, with a slight gain. On this trace v001's gate was already a small net positive at 128K, so the gate seldom turned out to be a cost.

## Hurt

- hit-rate (16K, both traces): A slight loss. At the small size, targets are usually full and the two-hit rule still applies. When a target does have a free slot, though, the item that promotes on one hit takes that slot. A later promotion into the same bucket then has to evict someone, and that starts a `target -> stash` cascade. Gating on whether the target has room right now ignores this delayed cost.

## Takeaway

Making the gate depend on occupancy mostly recovers v001's `web_0` 128K regression at a small cost at 16K. This is a net gain, but a small one. The `is_full()` check treats a free slot as cost-free, which holds in roomy caches. In tight caches a single-hit item can still take the last slot and push the swap onto the next promotion. A stricter condition, such as requiring more than one free slot, might keep the 128K gain without the 16K cost.
