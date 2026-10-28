# Variant v003 Insight

## Helped

- hit-rate (16K, `web_0` and `hm_0`): Gated stash→target promotees now land at the target's MRU end (`STASH_PROMOTION_TARGET_POS = Back`). An item that just earned two stash hits is therefore no longer the target's next eviction victim. At the small size, each target slot is contested, and in the parent the gate's filtering was thrown away by evicting the newcomer first. Keeping it resident recovers that value. The gain is clearest on `hm_0`. The hypothesis worried that pushing hot residents out faster would cost hits at 16K. That did not happen.
- hit-rate (128K, `hm_0`): The largest per-cell gain. A promotee at the MRU end is immediately the ClassicLRU `promotion_candidate_index()`, so its next hit moves it straight to probing instead of climbing first. On a trace with strong short-term reuse, this fast track to the probing tier pays off when there is capacity to hold the promoted items.

## Hurt

- hit-rate (128K, `web_0`): Essentially flat, with a marginal dip. With spare target room, the promotee was rarely the victim even at `Front`, so the placement fix has little to save here. The faster one-hit jump to probing can evict probing residents that would have stayed if the item had to climb first. The displaced target LRU resident goes to stash with its `lfu_counter` reset and must earn two fresh hits to come back.

## Takeaway

Where the gated item lands in its target matters as much as whether it is admitted. With the two-hit gate in place, `Front` placement had made the gate partly self-defeating. Moving only the stash→target placement to `Back`, while foreign-resident retries stay at the cold `insert_pos`, gives a net gain driven by the 16K cells and `hm_0`. The one flat cell (`web_0` at 128K) suggests the remaining cost is the item's now-immediate eligibility for probing, not the target placement. A follow-up could keep `Back` placement but require one more target hit before a fresh promotee can move to probing.
