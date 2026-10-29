# Variant v003 Insight

> Note: this insight is based on the hypothesis (H03) and on the parent and current fitness vectors. The overlay source files could not be listed or opened in this session, so I could not check the implementation against the hypothesis line by line. That includes the requirement for a conditional store instead of an unconditional one.

## Helped

- hit-rate (web_0, 131072): This cell gained the most. It is the cell H03 targeted. A non-hottest target hit now clears a pending hottest-hit count, so keys whose popularity flickers can no longer save up a "free" first hit and later trigger the full-probing cascade. As a result, fewer stable probing residents are displaced.
- hit-rate (other three cells): web_0/16384, hm_0/16384 and hm_0/131072 each rose a little instead of dropping. This refutes the risk H03 named, that small caches would lose keys because target residence there is too short for two hottest hits in a row. In practice, keys that really are hot reach the hottest position again quickly enough that the stricter rule rarely blocks them.

## Hurt

- per-op-time (get median): Get p50 rose by a few ns. H03 named exactly this as a refutation condition. The new check sits on the non-hottest target-hit arm, which is the most common hit path. Even a predicted-not-taken read-and-branch on `lfu_counter` adds cost there that the rare `is_hottest` arm never did.
- per-op-time (get tail): Get p99 rose sharply. This is more than a single well-predicted branch should explain. Possible causes are a store that is not reliably skipped, or the extra branch merging with the parent's already slow guarded cascade path. Neither can be confirmed without the source.
- per-op-time (put): Put p50 rose slightly and put p99 rose sharply, even though `put` was not touched. The likeliest explanation is benchmark noise or a shift in the environment, not this change. That also means part of the get-side regression may be noise.

## Takeaway

Making the probing guard truly "consecutive" works in the direction H03 predicted. It is a small refinement, not a step change: the gain sits mostly in web_0 at the large size, and no cell lost hit-rate. The cost is on the hot get path. Every non-hottest target hit now pays for a counter check, so median get latency moves even though the logic almost never fires. Unless that check can be removed from the common path, the hit-rate gain probably does not pay for its latency.
