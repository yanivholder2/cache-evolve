# Variant v002 Insight

## Helped

- hit-rate (16K and 128K, `web_0`): Both cells rose slightly, in the direction the hypothesis expected. Far-zone returns whose target is full no longer get pre-credited, so their first stash hit cannot promote them and evict a resident. That removes some of the stale-credit displacement that the longer `GHOST_FACTOR = 2` ring had added for `web_0`. The gain is small, though, and close to noise.
- hit-rate (mean): The summary rose by a margin that is effectively flat. The `web_0` gains roughly cancel the `hm_0` 128K loss, so the change mostly reshuffled where the gains landed rather than lifting the mean.

## Hurt

- hit-rate (128K, `hm_0`): This cell slipped slightly, which is the opposite of what the hypothesis predicted. The theory was that free target slots exist at 128K, so far-zone credit would be kept there. In practice, a key reaches the ghost only after the c7 rescue found both its target and probing buckets full. When it returns one to two stash-capacities later, its target is usually still full, so the far zone mostly denies credit. Some of those denied promotions, the ones that displaced a resident, were apparently paying off for `hm_0`'s long-tail returns.
- hit-rate (16K, `hm_0`): Flat. At 16K the targets are saturated, so far-zone hits almost always fail the occupancy check. Because far hits are rare there anyway, losing their credit changes little.

## Takeaway

The occupancy condition covers less than it seems to. `STASH_GATE_BYPASS_FREE_SLOTS = 2` already promotes on the first hit when a target is empty, so far-zone credit adds anything only when the target has exactly one free slot. In every other case the check simply removes credit. Combined with the c7 rescue, which already means ghost entries come from full neighbourhoods, the near/far split acts mostly as "drop far credit", not "drop far credit only where it harms." The data also contradicts the lineage rule that lost-slot items should never outrank residents: on `hm_0` 128K, letting far-zone returns displace residents was worth something. A next step should decide credit by the resident it would displace, for example its recency or hit count, rather than by whether a slot is free.
