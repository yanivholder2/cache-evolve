# Variant v002 Insight

> Note: I could not list or open the overlay source in this session. The only tools available were file read and write, with no directory listing, and guessed source paths did not resolve. The mechanism below comes from H02's specified changes. The diff itself was not checked, including whether a new per-slot "ghost-protected" bit was added or an existing reference bit was reused.

## Helped

- hit-rate: Keys admitted on a ghost hit now get a one-shot pass through the stash's second-chance scan, so they are no longer chosen as victims the first time the scan reaches them with counter 0. This let a few more returning keys survive until their first reuse. The gain over v003 is barely above zero, much smaller than the gain v003 itself got from MRU placement, and on one trace it is hard to tell apart from noise.

## Hurt

## Takeaway

On this trace, MRU placement alone already protected nearly all ghost-admitted keys. Few of them were being lost to the scan before their first reuse, so adding scan-skip protection on top of MRU placement gives almost no further return. That answers v003's open question about protecting ghost hits from the next scan pass: survival in the stash is not where the remaining misses come from. Stash-victim selection for ghost hits looks tuned out. Further gains more likely lie in the ghost window's reach (reuse distances beyond one cache's worth of evictions) or outside the stash path. Any added metadata bit and the single-trace, single-size evaluation are not captured by this fitness vector.
