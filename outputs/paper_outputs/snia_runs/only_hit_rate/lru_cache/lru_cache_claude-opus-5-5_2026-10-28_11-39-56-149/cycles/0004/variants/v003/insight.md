# Variant v003 Insight

## Helped

- hit-rate (web_0 @ 16384): This is the cell the hypothesis targeted, and it moved the predicted way. At this size the window holds only about 160 entries. A web_0 block re-referenced inside that window used to reach the admission duel with a doorkeeper-deflated estimate and lose to a probation victim with stale counts. With `win_ref` it skips the duel, so this recency-driven reuse now stays resident. The gain is modest, which suggests that only part of the cell's earlier loss came from the duel.
- hit-rate (hm_0 @ 131072): Roughly flat, with a slight uptick. With the larger window and probation segment, the few bursty hm_0 blocks that pass on `win_ref` displace probation entries that would mostly have aged out anyway.

## Hurt

- hit-rate (hm_0 @ 16384): This is the largest move in the variant, and it falsifies the hypothesis's key premise that hm_0's one-shot blocks never earn the flag. Any second touch during the window stay sets `win_ref`, whether a `get` hit or a `put` on an existing key. On hm_0 that covers back-to-back read/write or multi-request bursts on the same block, and such blocks are then dead. These burst-then-dead blocks now bypass frequency admission and evict probation victims with real long-range reuse. That largely undoes the one-shot filtering that v002's doorkeeper and dedup changes had won in this tight-capacity cell.
- hit-rate (web_0 @ 131072): Here the window is about 1% of capacity, roughly 1300 entries. A larger window gives many more blocks a chance to collect one in-window hit, including blocks with only short-lived reuse. The parent insight noted that at this size the duel was already mostly correct. Overriding it on a single hit swaps frequency-earned probation entries for burst-only candidates.

## Takeaway

A single in-window re-reference is too weak and too common a signal to justify an unconditional admission bypass. hm_0's scan traffic often arrives in short bursts, so the flag leaks exactly the blocks v002's sketch changes were filtering. The flag fires more often as the window grows, which is why the net effect is negative at large web_0. The small-size web_0 gain shows that in-window reuse carries real information. However, that information should only nudge the frequency duel (for example, as a bounded bonus or a tie-breaker), not replace it. Using it as a hard override costs more on hm_0 than it buys on web_0.
