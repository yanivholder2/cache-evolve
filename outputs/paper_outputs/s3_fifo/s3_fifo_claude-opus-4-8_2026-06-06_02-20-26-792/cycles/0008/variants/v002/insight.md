# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Deleting the weak-ghost `else` branch in `evict_small()` — so a `freq <= 1` small-queue victim is dropped with no `GhostKey { from_main: false }` record — destroys the cheap second-chance signal those weak ghosts carried; a reaccessed one-hit-wonder now re-enters the small queue and must re-earn `freq > 1` to promote instead of re-admitting straight to main via the in-ghost path, collapsing hit-rate by roughly a fifth.

## Takeaway

The premise that weak small-evicted ghosts are low-value capacity-wasters is wrong on this trace: those `from_main: false` records are the dominant re-admission source, and removing them strips the small queue's only memory of a key seen once-then-evicted. Reallocating the entire `main_size` ghost budget to proven main-evicted keys does not relax the constraint for strong keys — it starves the far larger weak-reuse population. Source-aware *trimming* (cycle 0007's keep-strong-longer ordering) helps at the margin precisely because it still ghosts weak keys; *not ghosting them at all* is a severe regression.
