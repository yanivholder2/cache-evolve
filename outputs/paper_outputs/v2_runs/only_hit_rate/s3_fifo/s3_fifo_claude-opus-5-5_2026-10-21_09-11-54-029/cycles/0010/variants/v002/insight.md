# Variant v002 Insight

## Helped

- hit-rate: Halving the small queue (`capacity / 40`, 51 slots at 2048) gives about 51 slots back to main, which clear-on-pass SIEVE uses at least as well as the admission filter did, so hit-rate went up slightly. The gain is marginal and comes from one trace at one size. Because `ghost_ring` is sized `main_size.max(1)`, the ghost also grew by the same 51 slots. That softens the main cost of the shorter probation window: a key that needs two hits within 51 small-queue insertions to graduate, and is ejected before getting them, now has a better chance of being found in the ghost and sent straight to main when it comes back.

## Hurt

(No measured dimension regressed. Only hit-rate is in the fitness vector. The change is a single size constant: it updates `try_new`, the lockstep `RefS3Fifo::new` and the size test, and leaves the eviction, graduation (`freq > 1`), ghost re-admission and fingerprint code unchanged. So none of the unmeasured cost dimensions should have moved either.)

## Takeaway

Under clear-on-pass SIEVE, the 5% small-queue split tuned for FIFO-reinsertion main is not optimal, but it is close. Going to 2.5% gives only a very small positive change, which suggests the split is near flat. Since main and ghost grew together, this variant cannot tell whether the gain came from extra main residency or from ghost reach. Separating the two would require giving the ghost its own size budget. A further cut (for example `capacity / 80`) is also worth testing to see whether the curve keeps rising or turns down.
