# Variant v003 Insight

## Helped

- hit-rate: the small-cache cells improved on both traces, with web_0 small gaining the most. At 16384 entries the window is small, so a candidate only gets hit in the window if it is re-referenced soon after it arrives. That makes the `reused` flag a strong reuse signal there. Letting those candidates win ties against a cold sampled probation victim moves real short-reuse keys into main, where strict `>` would have dropped them to stale entries.
- hit-rate: hm_0 small, the cell the hypothesis was aimed at, moved in the right direction but only slightly. The rule only applies on exact estimate ties, and each window hit also bumps the sketch (double counting), so many reused candidates already beat the victim under `>`. The tie rule therefore has little extra to admit.

## Hurt

- hit-rate: the large-cache cells regressed slightly on both traces, enough to cancel the small-cache gains. At 131072 entries the window holds many more entries for longer, so far more candidates pick up a window hit before reaching the window LRU. The flag never clears during window residency, so one early hit is enough. For a large share of candidates the gate then acts like the global `>=` tie-break that cycle 0004 already found costly on web_0 large. Victims at the probation head with the same never-aged estimate are long-resident keys that were worth keeping more often than the newcomer.

## Takeaway

Gating the admission tie-break on "hit while in the window" is a good reuse signal only when the window is short. Its precision falls as the window grows, so in this lineage it trades small-cache gains for large-cache losses, and the summary barely moves. If reuse evidence is to drive admission, it should depend on the window's size: for example, require the hit to happen near the window LRU, or count window hits relative to window residency. A sticky boolean whose meaning drifts as the window climber resizes the window does not work. hm_0's deficit is still not explained by tie-breaking at the admission contest.
