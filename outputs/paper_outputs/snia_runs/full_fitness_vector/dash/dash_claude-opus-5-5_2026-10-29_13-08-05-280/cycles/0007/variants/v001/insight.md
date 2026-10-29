# Variant v001 Insight

## Helped

- per-op-time (get tail): Get p99 went down a little. This fits with removing the `full_target_guard` field read and its branch from the stash-hit path. The full-probing cascade and the `PROBE_WINDOW` arm were left alone, so their cost is still there and the gain stays small.
- hit-rate (web_0 small cell): The 16384 cell moved up by only a rounding-level amount. Always requiring the third stash hit before promoting into a full bucket barely changes web_0 at this size.

## Hurt

- hit-rate (hm_0 small cell): The 16384 cell went down instead of moving toward cycle 0006 v001's value. At small sizes, v002's gate lowered the full-bucket stash promotion bar to two hits. Under the full-probing cascade (no swap), that lower bar was helping hm_0. Forcing three hits delays promotion of hm_0's recurring stash keys, so they get evicted from the stash before they reach the target bucket.
- hit-rate (summary): The small-cache gain the hypothesis predicted did not happen. The two 131072 cells did not move, as expected, because the gate only acted at small sizes. So the hm_0-small loss pulls the aggregate slightly below the parent.

## Takeaway

This variant meets the hypothesis's own refutation condition. With v002's full-probing cascade, dropping the size gate leaves the small-cache cells at v002's level or worse. So cycle 0006 v001's small-cache gain came from the swap, or from the swap combined with the always-three-hit rule, not from removing the gate alone. The gate was useful where there was no swap. Removing it is a small cleanup on get p99 but costs hit-rate. To get v001's small-cache gain without the swap's tail cost, look for a cheaper version of the swap's effect, not for changes to the gate. Note: this insight is based on the hypothesis, both fitness vectors, and the parent insight. The overlay source files could not be listed in this session, so the code-level reasons assume H01 was implemented as written. The one-ns rise in put p50 is on an unchanged path, so treat it as noise.
