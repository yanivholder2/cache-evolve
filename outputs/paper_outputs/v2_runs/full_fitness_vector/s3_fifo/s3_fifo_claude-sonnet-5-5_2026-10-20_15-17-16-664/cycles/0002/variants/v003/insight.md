# Variant v003 Insight

## Helped

- hit-rate: A separate bounded main-eviction ghost lets recently evicted main keys re-enter main on a miss. The original demotion ghost stays unpolluted, so the one-hit-wonder filter is intact, and the gain is small but positive.
- per-op-time (get): The measured get latency fell versus the parent. The change touches only the miss and insertion path, so this is likely run-to-run noise or layout effects, not a real effect of the design.

## Hurt

- metadata-size: Struct-level size grew because of the added second ghost FIFO and its bookkeeping. Per-entry size is unchanged, since the key-only entry representation was reused.
- per-op-time (put): Put p50 rose because each put now probes a second ghost on a miss and records main evictions into it. Put tail latency improved, which suggests the extra work is cheap and the tail gain is mostly noise.

## Takeaway

Splitting main-eviction history from small-queue demotion history fixes the filter dilution seen when both shared one ghost, and it yields a modest hit-rate gain. The cost is a fixed struct-level overhead and a slower typical put, not a larger per-entry footprint. The gain is small, so a ghost sized at about 25% of main is a lightly tuned point, not a clear win.
