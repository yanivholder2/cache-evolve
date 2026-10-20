# Variant v002 Insight

## Helped

- hit-rate: Tagging ghosts with `from_main` and trimming via `evict_ghost_one()` — which scans from the tail for the oldest weak (`from_main == false`) small-evicted record and only falls back to `pop_back()` when none remain — keeps strong main-evicted history resident longer, so a few more proven-valuable keys survive in the ghost window to re-admit directly to main on reaccess.

## Hurt

## Takeaway

Prioritizing ghost retention by source — sacrificing weak one-hit-wonder history before strong main-evicted history within the same `main_size` budget — is a real, valid lever and it does lift hit-rate without touching ghost depth or partition ratio. But the gain is tiny on this LIRS trace: the weak-vs-strong eviction order only matters when the ghost queue is full *and* a displaced strong key would have been reaccessed before its slower-arriving weak neighbor expired, a narrow window that rarely binds here. Reordering retention priority helps only at the margin; the deeper constraint remains raw ghost capacity, not which source gets dropped first.
