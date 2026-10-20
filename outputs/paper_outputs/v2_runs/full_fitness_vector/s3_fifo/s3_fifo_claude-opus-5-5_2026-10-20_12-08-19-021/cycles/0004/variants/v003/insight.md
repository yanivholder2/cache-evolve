# Variant v003 Insight

## Helped

- per-op-time (`put`): The modest gain at both p50 and p99 comes from removing machine work, not from removing hashing. The likely causes are two: the `% self.ghost_ring.len()` in `ghost_push` became a compare, and the `#[inline]` hints on `evict_main` / `evict_small` / `evict_small_inner` let those functions fold into `put`. The ring length is `main_size` (not a power of two at this capacity), so the old wrap was a real integer division on every small-queue eviction that ghosts a victim. The `alloc_with_key` change only builds the `Entry` directly in its slab slot instead of building it first and then moving it in, which is a small saving at most. This variant cannot tell the three contributions apart.

## Hurt

- per-op-time (`get`, the summary): The `get` body is byte-identical to v002's, and so is the `framework::Cache` wrapper. The small upward move is therefore noise, or a code-layout side effect of inlining the eviction functions into `put`. It is not a real cost of this change. Note that the summary dimension tracks `get`, so a `put`-only refactor cannot improve the headline number.

(hit-rate and metadata-size did not move. An identical hit-rate confirms that the refactor is policy-neutral as required: `fp` stays `FxHash | 1`, and the eviction and ghost logic are unchanged. `Entry` and the struct fields did not change, so both size rows are the same.)

## Takeaway

H03 miscounted v002's miss path. v002 already computed `fp` only once (inside `ghost_take`), did one key clone (in `alloc`), and made two `index` hash operations (probe plus insert). After this change the counts are the same: one `fp`, one clone, two `index` ops. So "single-hash `put`" removed no hashing at all, and the `put` gain belongs to the modulo-to-compare wrap and the eviction inlining. Before attributing future `put` latency work to hashing, count hashes and clones in the parent code. Also, a truly single-probe miss needs a different structure, such as a precomputed-hash/raw-entry index, not a reshuffle of helper signatures.
