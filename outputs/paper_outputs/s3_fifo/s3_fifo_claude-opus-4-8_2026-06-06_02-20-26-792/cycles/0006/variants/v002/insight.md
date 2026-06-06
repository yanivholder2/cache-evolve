# Variant v002 Insight

## Helped

- hit-rate: Ghosting the `freq == 0` main victim in `evict_main`'s `else` branch (mirroring `evict_small`'s capacity-bounded `push_front`) lets a previously-promoted, briefly-cold key re-admit straight to main on reaccess via the existing `put()` in-ghost path, recovering a reuse signal the parent silently dropped.

## Hurt

## Takeaway

Recording main-evicted victims — a population that earned promotion at least once — in the ghost queue is a genuinely new lever versus prior cycles that only ever ghosted the weak small-evicted one-hit-wonders, and it does lift hit-rate. But the gain is slim: on this LIRS trace few main residents cool to `freq == 0` and then get reaccessed inside the `main_size`-bounded ghost window, so widening the ghost *source* helps only at the margin without touching ghost depth.
