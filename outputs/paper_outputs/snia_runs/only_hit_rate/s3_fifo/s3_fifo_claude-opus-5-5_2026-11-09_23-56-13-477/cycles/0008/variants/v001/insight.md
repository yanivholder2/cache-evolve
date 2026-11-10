# Variant v001 Insight

## Helped

- hit-rate (hm_0 @ 16384): A small gain. Main-victim ghosts no longer advance `next_stamp` or take slots in the small-victim ghost, so the cycle-0005 freq-0 recency window is back in place. hm_0's quick small→ghost returnees reach main directly again, and a few hit main victims still get their own one-small-queue-sized memory.
- hit-rate (hm_0 @ 131072): A slight gain. hm_0's re-referenced main victims seem to come back quickly, so the new `main_ghost` budget (`max(1, small_size)`) catches them even though it is much smaller than the shared `main_size` ghost they used to compete for.
- hit-rate (web_0 @ 16384): Flat, with only a negligible uptick. Putting the parent's pre-0007 stamp clock and ghost bound back did not recover the small-size loss that cycle 0007 blamed on coupling. That coupling was at most a minor cause.

## Hurt

- hit-rate (web_0 @ 131072): This cell gave back the parent's gain, and then some. In the parent, hit main victims could fill up to `main_size` ghost slots. Here they are capped at `small_size`, about a third of that (small is a quarter of capacity, main three quarters). web_0's long-reuse main victims age out of `main_ghost` before they return, so they have to re-qualify through small again.

## Takeaway

Decoupling the main-victim ghost does what it was meant to do: it protects the freq-0 age window and is neutral-to-positive for hm_0. But capping that ghost at one small-queue's worth of keys removed the only mechanism that helped web_0 at the large size. web_0's main victims return over a horizon closer to `main_size` than to `small_size`. And the small-size loss on web_0 was not fixed by restoring the clock, so the parent's diagnosis of that loss was mostly wrong. The open lever is the `main_ghost` budget. A budget that scales toward `main_size` while staying decoupled is the direct test of whether the large-size web_0 gain can be kept without the shared-clock side effects.
