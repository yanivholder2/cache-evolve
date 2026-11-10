# Variant v003 Insight

## Helped

- hit-rate: On hm_0, both cache sizes gained a little. A 10% small queue sends one-hit-wonders to the ghost queue sooner, which leaves more room in main for the reused working set. The bigger ghost queue (now sized to the 90% main) brings back keys that were demoted early.

## Hurt

- hit-rate: web_0 at the large size (131072) went down, which is the opposite of what H03 predicted. With `freq > 1` still required to graduate, a key now has to be accessed twice within a much shorter FIFO window. Keys with medium reuse distances get ghosted, and each one costs an extra miss before a ghost hit re-admits it to main.

## Takeaway

Changing the small/main split from 25/75 to 10/90 on its own left aggregate hit rate essentially flat: small gains on hm_0 were cancelled by a loss on web_0. The prediction that a larger main helps most at large sizes did not hold. The 10% split only helps when graduating is easy, so `freq > 1` with `MAX_FREQ` 3 (left unchanged here) likely holds it back. Changing the small-queue size together with the graduation threshold, or using a per-trace small-queue size, looks more promising than changing the ratio alone.
