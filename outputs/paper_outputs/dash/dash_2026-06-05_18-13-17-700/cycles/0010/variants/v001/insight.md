# Variant v001 Insight

## Helped

- (no dimension improved) The grafted breadth-of-homes rescue did fire — relocating warm probing victims into their second probing home (`probing_alt = (target_idx + 2) % segment_size`) before demotion — but on top of the v002 ghost gate it bought no net residency that the ghost fast-track wasn't already covering.

## Hurt

- hit-rate: Adding the probing rescue hop to the ghost-winner parent nudged hit-rate down rather than up — the two seams are not orthogonal in practice. By keeping warm victims resident in an alternate probing slot instead of demoting them, the rescue holds onto keys that the parent would have dropped into the ghost ring; that suppresses the very dropped-then-recurring signal the v002 ghost gate depends on, so the rescue cannibalizes the parent's gain instead of stacking with it.

## Takeaway

v003's probing rescue and v002's ghost gate compete for the same warm-victim population rather than splitting it: the rescue's whole purpose is to prevent the eviction that the ghost ring needs to record, so grafting them together starves the ghost fast-track of its inputs and the combination lands slightly below the v002 winner alone. The cross-cycle "do they compound" question resolves to overlap, not synergy.
