# Variant v001 Insight

## Helped

(none — no hit-rate cell moved relative to cycle-0006 v001)

## Hurt

- hit-rate: Neutral, not hurt. The change did land in the overlay: `FOREIGN_RETRY_REQUIRES_FREE_SLOT = true` and the full-target check comes before the retry `put_at` in `demote_from_normal_bucket`. Yet all four trace×size cells, `web_0` 128K included, match the parent exactly. So the cell it was aimed at did not recover, and the `hm_0` gains stayed only because nothing changed.

## Takeaway

When the target is full, sending the displaced foreign resident straight to the stash produced the same results as the parent. The most likely reason is that the parent's path already did the same thing. The foreign retry uses the cold `Front` position, so when a full target accepts the item at its LRU end, the item itself may well be the one evicted into the stash. The rationale's picture, where a target resident gets cascaded out, would then not be what happens with this bucket's `put_at` semantics. That would make the guard a no-op. I could not check this against the bucket source from here, so it remains an inference from the identical results. Whatever the cause, the "foreign resident cascades a target item into the stash" path is not the source of the `web_0` 128K giveback. Later hypotheses should look for that cost somewhere else, such as ghost churn from the higher number of probing evictions, and stop changing who leaves on the foreign-retry path.
