# Variant v001 Insight

## Helped

- hit-rate (128K, `web_0`): This is the cell the hypothesis aimed at, and it moved up a little. When a stash item gets its first hit and its target bucket has at least `STASH_GATE_BYPASS_FREE_SLOTS` free slots, it now skips the two-hit gate and goes straight to the target's MRU end. Big caches have more buckets with spare room, so `web_0` items that are reused only once while in the stash now reach the target and stay there. Before, the parent's gate held them back. Because the target had room, these promotions evict nobody.
- hit-rate (16K, `hm_0`): A very small gain. In the few small-cache buckets that still have two free slots, first-hit promotion now fires and fills space that would otherwise sit empty.

## Hurt

- (none) hit-rate (16K, `web_0`) and hit-rate (128K, `hm_0`) did not change. The 16K `web_0` cell is the one where cycle-0003 v002's `!is_full()` bypass had cost hits. It held steady here, because the two-free-slot threshold switches the bypass off before a one-hit item can take a contended bucket's last slot. On `hm_0` at 128K, the parent's `Back` placement plus quick promotion to probing already caught the reuse. Letting items in one hit earlier added nothing.

## Takeaway

A gate bypass that depends on free space works if the threshold leaves the last free slot for items that passed the full gate. Requiring two free slots, not just "not full", kept v002's 128K `web_0` gain without its 16K loss. But the gain is small and comes from a single cell. With the 2-slot buckets used in testing, the bypass only fires when the target is empty, so it rarely triggers. Further gains will probably come from a different mechanism, not from tuning this threshold.
