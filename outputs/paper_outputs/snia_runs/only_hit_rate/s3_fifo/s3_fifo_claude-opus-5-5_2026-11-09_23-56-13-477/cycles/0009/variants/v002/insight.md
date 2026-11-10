# Variant v002 Insight

## Helped

- hit-rate (hm_0 @ 16384): A small gain. When the cache is small, `small_size` is only a few thousand keys, so the parent's `main_ghost` cap pushed out hit main victims early. Moving the cap to `main_size` lets more of hm_0's re-referenced main victims go straight back into main instead of re-qualifying through small. The decoupled `next_stamp` clock keeps the freq-0 window intact.

## Hurt

- hit-rate (web_0 @ 16384): A small loss. With the larger budget, main victims ghosted long ago still count as main-ghost returnees. Each one is pushed into main with freq 0 and forces an `evict_main`. At this size main is tight, so these older returnees push out residents that web_0 would have reused sooner.

## Takeaway

Making the main-victim ghost three times larger left both large-size cells exactly where they were, web_0 @ 131072 included. That refutes H02's premise and the parent insight's diagnosis: at the large size, the `main_ghost` budget is not the limiting factor. Either hit main victims are too few to fill even a `small_size` budget, or the ones that return come back within that horizon anyway. The budget only matters at the small size, and there it trades hm_0 against web_0. The leftover web_0 large-size gap has to come from something other than how long hit main victims are remembered. Two candidates: which victims get ghosted at all (the `main_hit` gate), or how returnees are admitted (straight in with freq 0).
