# Variant v001 Insight

## Helped

- **hit-rate**: Stash-promoted items inserted at MRU position (Back) become immediate promotion candidates on next access, accelerating proven-hot items through the tier hierarchy faster than LRU-positioned inserts.

## Hurt

(none)

## Takeaway

Inserting stash hits at the MRU end of the target bucket rewards their proven hotness: these items have already survived demotion and returned, making them worthy of immediate re-consideration for probing. Front insertion forced them to earn their way back through the target tier; back insertion recognizes their history and unlocks them for faster tier climbing.
