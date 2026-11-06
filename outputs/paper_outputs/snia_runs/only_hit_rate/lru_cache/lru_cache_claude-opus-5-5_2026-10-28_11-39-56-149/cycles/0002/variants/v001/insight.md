# Variant v001 Insight

## Helped

- hit-rate (hm_0 @ 16384): This is the only cell where the hypothesis held. On hm_0 at the small size, many blocks come back after they have left the short probation list but while they are still in the capacity-sized ghost. Sending those returns straight to protected kept them resident, which v001 could not do.

## Hurt

- hit-rate (web_0, both sizes): A ghost hit puts the key at protected MRU, and that forces `demote_protected_lru` whenever protected is full. On web_0, a key that comes back once from the ghost seems to be a weaker reuse signal than a key re-referenced while still resident. These one-time returnees pushed established protected entries back into probation, where they were evicted next. The loss is largest at 131072, where the ghost window (`ghost_cap = capacity`) is biggest and so admits the most keys to protected.
- hit-rate (hm_0 @ 131072): A small loss. At this size v001's protected segment already held most of the reused working set, so ghost admissions mostly displaced useful protected entries instead of recovering missed reuse.
- hit-rate (summary): Overall, the web_0 losses and the small hm_0 @ 131072 loss outweigh the hm_0 @ 16384 gain.

## Takeaway

On these traces, a ghost hit is not as strong a signal as a resident re-reference. Giving it immediate protected status breaks the SLRU guarantee that protected holds only keys seen twice while cached, and it hurts most when the ghost window grows with capacity. A better follow-up would treat ghost hits more weakly. Options: admit them to probation MRU, or make them win only against the probation victim (TinyLFU-style). Another option: shrink or adapt the ghost window, perhaps to the probation size, instead of fixing it to the full capacity. Metadata size and per-op time were still not measured, so the cost of roughly one key of extra metadata per slot is not yet known.
