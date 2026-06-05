# Variant v001 Insight

## Helped

- hit-rate: Giving each overflowing stash victim one cuckoo-style hop into its salted alternate bucket (`stash_insert` relocating to `stash_alt_index` when distinct and non-full, plus `promotion_get` checking both homes) rescues a handful of cold keys that the parent dropped on primary-bucket collision, yielding a barely-positive nudge.

## Hurt

- hit-rate: The relocation seam barely registers on top of the winner — the dwell-time extension the stash hop buys mostly covers keys the protected-tier/touch-counter config was already capturing, so the second home displaces stash residents of comparable recency and nets almost no new hits.

## Takeaway

This is the direct test of whether the cold-key dwell seam (v002's stash relocation) and the protected-tier seam (the v001 winner) are additive or chase the same key pool — and the flat result says they overlap, not stack. The one-hop relocation that helped over v002's weaker stash baseline does not survive on top of the proven best: once the two-speed promotion and 3:1 geometry already keep warm keys resident, extending a colliding stash victim's life rescues keys the cache reclaims through promotion anyway. Combining independently-discovered slivers does not compound them here.
