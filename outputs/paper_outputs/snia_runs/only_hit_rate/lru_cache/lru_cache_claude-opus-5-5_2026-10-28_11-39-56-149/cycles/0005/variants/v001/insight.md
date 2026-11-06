# Variant v001 Insight

## Helped

- hit-rate (web_0 @ 131072): A marginal gain, within the range of noise. At this size each climb step is at most 256 entries, a tiny fraction of the cache. The extra rounds from a 20,000-access period therefore cost little even when the climber flips direction on a noisy sample. Adjusting more often lets it follow web_0's phase changes slightly better.

## Hurt

- hit-rate (web_0 @ 16384): This was the cell the hypothesis expected to gain most, and it fell clearly. Window resizes are lazy and happen only on full-arena puts. Each put can also evict the candidate instead of the victim. So a 20,000-access round, only about 1.2x capacity, often ends before the window has filled to its new `win_cap`. That round's sampled hit rate then reflects a half-applied move. With flip-on-drop, this transient reverses growth that would have paid off later, and the window stops short of the size v002 reached.
- hit-rate (hm_0 @ 16384): A small loss. Shorter samples on a scan-heavy trace give noisier hit-rate comparisons, so the climber flips more often. The 2% per-round decay also shrinks the step 2.5x faster per access, so step resets come sooner. Together these leave the window wandering above its 1% floor and let scan traffic into a window that should have stayed small.
- hit-rate (hm_0 @ 131072): A small loss from the same direction-flipping. Here a 20,000-access sample is much smaller than the cache, so one round barely changes what the cache holds. The climber is reacting to sample noise, not to the effect of its own last move.

## Takeaway

Making the sample period shorter did not make the climber adapt faster. Window resizes only take effect as misses arrive, so at 16384 a 20k round is too short for a move to show up in the hit rate before the next comparison. The climber ends up judging moves that have not landed yet. This rules out the period cap as the source of v002's web_0@16384 gain, which points to the 256-entry step cap, or to v002's 50k period being about the right length. The next isolating test should vary the step cap with the period held at 50,000. The period should probably stay tied to how long a resize takes to fill (a multiple of capacity), not to a fixed access count. The new `climb()` log line should be checked to confirm that `win_cap` stalls or oscillates at 16384.
