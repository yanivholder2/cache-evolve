# Variant v001 Insight

## Helped

- **hit-rate**: Tier-skipping stash hits directly to probing (instead of target) combined with MRU positioning accelerates access to the hottest tier; items proven hot enough to return from stash bypass the intermediate validation step and reach their final promotion destination faster.

## Hurt

(none)

## Takeaway

Combining tier-skipping with MRU positioning validates the synergy hypothesis: when a stash hit occurs, promoting directly to probing with Back insertion recognizes that the item has already demonstrated its worth (survival through demotion) and is now immediately ready for hot-tier placement. This eliminates an unnecessary stall in the target tier, allowing proven-hot items to settle into their peak tier position on the same access rather than requiring another traversal.
