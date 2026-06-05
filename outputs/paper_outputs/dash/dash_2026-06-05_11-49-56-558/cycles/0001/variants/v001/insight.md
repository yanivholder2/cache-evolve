# Variant v001 Insight

## Helped

- hit-rate: Inserting stash-promoted items at the MRU end (`InsertPosition::Back`) of the target bucket means a recently-reactivated cold item is immediately the hottest resident and qualifies for probing-tier promotion on its very next access, rather than having to crawl from the LRU end through a full bucket's worth of accesses first.

## Hurt

_(none)_

## Takeaway

Changing a single call-site in `Segment::get` from `InsertPosition::Front` to `InsertPosition::Back` collapses the "cold re-entry penalty" for stash hits from O(bucket-size) accesses to one: the item lands at the MRU end on reactivation and is immediately eligible for promotion to the probing tier. This confirms that stash re-entry position is a high-leverage parameter in the Dash promotion ladder — the baseline's conservative Front insertion was unnecessarily delaying items that had already proved themselves by surviving in the stash.
