# Variant v002 Insight

## Helped

(none — no dimension improved in a way the change explains. `put` latency moved slightly, but `put` never runs the changed stash-hit branch, so that is run-to-run variation. metadata-size did not move: only a const and an extra `retry_pos` parameter were added, with no new fields or types.)

## Hurt

- hit-rate: A stash-hit promotion now lands at the target's MRU end (`STASH_PROMOTE_POS = Back`). In a 2-slot target, the resident that gets pushed to the LRU end, and is evicted on the next insert, is now the one that had been hit in place. A one-time stash re-read gets ranked above an item with proven target reuse. Also, the promoted item is already the hottest, so one more target hit sends it straight into probing. That pushes probing residents (the hottest tier) down the cascade sooner. Hit rate fell a little below v001 instead of rising. This is the "faster path into probing churns probing" outcome the hypothesis flagged as possible.
- per-op-time: The `get` tail grew while the `get` median barely changed. That matches a few gets now doing more work: the earlier target→probing promotion means more `promote_to_probing` evictions, each followed by a one-more-chance retry and possibly a cascade into the stash. The median `get` (a hit in place, or a miss) is untouched.
## Takeaway

v001 gained hit rate by placing items at the MRU end on the way down (demotion into the stash). Applying the same idea on the way up (stash→target promotion) does not carry over. In tiny target buckets, Back placement turns one stash hit into the right to evict an item already hit in the target, and it shortcuts the "climb within the bucket" step that kept probing selective. Front placement for stash promotions works as a useful admission filter. So further gains are more likely from things like requiring a second hit before promotion into probing than from making stash promotions land hotter.
