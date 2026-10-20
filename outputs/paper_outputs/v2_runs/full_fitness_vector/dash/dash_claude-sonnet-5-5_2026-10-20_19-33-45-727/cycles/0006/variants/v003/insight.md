# Variant v003 Insight

(Note: I could not list or read the overlay source files in this session, so this is based on the hypothesis and the fitness vectors only.)

## Helped

- per-op-time: Both get and put latency fell well below the parent. The hypothesis left get untouched, so the get gain is probably measurement or environment variation and not an effect of the doorkeeper. The put gain could be real, because declined first-touch inserts skip the insert and eviction work.

## Hurt

- hit-rate: The fingerprint ring declines the first insert of every key, and a 64-entry ring per segment is small. Recurring keys on the LIRS trace are likely overwritten before their second touch, so they are never admitted. That costs hits the baseline got by inserting immediately, and it is the opposite of the intended gain.

## Takeaway

Moving the second-hit filter out of the items and into a small fixed ring kept metadata-size at the parent's value. It did so by making the filter lossy, and the lost hits outweighed the evictions it avoided. Items that were seen once but are reused soon after, as in LIRS-style traces, need a larger or better-aged doorkeeper to survive until their second touch. The latency drop should not be credited to this change until get timing is re-measured against the parent.
