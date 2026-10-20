# Variant v002 Insight

> Caveat: this session only had a file reader, with no directory listing, and I could not find the overlay source or the `h02_split_report` test output under any path I tried. The hypothesis requires this insight to quote the settled `small_size` for step 1 vs step 2. **I cannot quote those numbers.** They need to be pulled from the eval logs before this variant's result is treated as settled. The interpretation below rests on the hypothesis text and the two fitness vectors only.

## Helped

(Nothing improved.)

## Hurt

- per-op-time (`put`): On a ghost hit, the shrink-by-two change replaces the single `if` fix-ups with `while` loops. One `put` can now evict up to two entries from `small`, and then from `main` as well. This plausibly accounts for some of the worse `put` tail. It cannot account for the size of the jump, because ghost hits are only a fraction of `put`s.
- per-op-time (`get`, the summary): Nothing in H02 touches the `get` path. The lookup, the slot layout and `Entry` are all unchanged, and `metadata-size` (including the `S3Fifo` header) is identical. So the policy change does not explain the `get` regression. It points to noise in the benchmark or a change in the environment between cycles. Re-run it before charging this to the change.

## Neutral

- hit-rate: This is identical to the parent at the reported precision. That is the "flat" outcome the rationale predicted. Either the `capacity/64` floor is what binds, or a faster shrink only gets `small` to the same equilibrium sooner. Either way, on this trace the shrink:grow ratio does not set where the split settles. The `settled_small`/`floor_blocks` figures from `h02_split_report` would tell these two cases apart.
- metadata-size: Unchanged, which fits `shrink_step` being a `#[cfg(test)]` field and `SHRINK_STEP` being a `const`.

## Takeaway

Doubling the ghost-hit shrink step did not change hit-rate at all. Like damping the grow-small trigger (0006 v003), it is inert, so on workload-a neither side of the adaptive trigger ratio is the lever. That makes the `capacity/64` floor (or the pinned ghost capacity) the likely binding limit. The next policy hypothesis should move the floor or the ghost sizing, not the step sizes. If `h02_split_report` shows `settled_small` at the floor for both steps, that confirms it. The `get` slowdown has no code-level cause in this diff and should be re-measured, not attributed to the change.
