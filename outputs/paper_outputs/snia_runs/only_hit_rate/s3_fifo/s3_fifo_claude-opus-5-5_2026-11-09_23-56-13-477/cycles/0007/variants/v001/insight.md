# Variant v001 Insight

## Helped

- hit-rate (web_0 @ 131072): This is the cell the hypothesis targeted, and it finally moved the right way after several cycles of no change. The gain is only marginal, though. Main victims that had been hit and then got a freq-1 ghost do return within the ghost window and skip small's two-reuse filter. But there are too few of them to matter, which suggests long-reuse keys that fall out of main are not what limits web_0 at this size.
- hit-rate (hm_0 @ 131072): A slight gain for the same reason. At the large size the ghost bound (`ghost_live <= main_size`) is wide enough in absolute terms that a hit main victim is often still ghosted when it returns, so it goes straight back into main.
- hit-rate (hm_0 @ 16384): Essentially flat with a negligible uptick. The ghost FIFO is small at this capacity, so few main-victim ghosts survive until their key comes back.

## Hurt

- hit-rate (web_0 @ 16384): A slight regression. Main-victim ghosts share the one ghost FIFO, so they push small-victim ghosts out sooner. They also advance `next_stamp`, which shrinks the effective freq-0 recency window from cycle 0005 (measured in ghost insertions). Some quick small→ghost returnees that the parent sent directly to main now age past the window and have to go back through small.

## Takeaway

Giving hit main victims a freq-1 ghost is a valid mechanism with a near-zero effect. It nudges the large size up and the small size down, and the summary is essentially unchanged. The cost comes from coupling: main-victim ghosts take space in the shared ghost FIFO and advance the same stamp clock that drives the parent's age gate. A follow-up should give main victims their own ghost budget, or stop advancing stamps on main→ghost inserts, before deciding whether this signal is worth keeping. Either way, the small gain at web_0 @ 131072 suggests its plateau has a cause other than main-victim re-qualification.
