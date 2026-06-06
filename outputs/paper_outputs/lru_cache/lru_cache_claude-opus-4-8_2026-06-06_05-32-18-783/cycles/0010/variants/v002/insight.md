# Variant v002 Insight

## Helped

- (none) — neither hit-rate nor metadata-size moved, and the targeted dimension, per-op-time, did not improve.

## Hurt

- per-op-time: Deleting the two trailing `set_prev_link(NIL)`/`set_next_link(NIL)` self-writes from `unlink` made both get and put p50/p99 rise above the parent — the change the hypothesis predicted would *lower* latency instead regressed it, because the stores it removed were already dead-code-eliminated by codegen and dropping them only perturbed register/instruction scheduling on the link-surgery hot path.

## Takeaway

The two NIL self-writes in `unlink` were not real work at runtime: the optimizer already elided them given every caller's immediate `push_back` overwrites both link words. Removing them at the source bought nothing and disturbed scheduling enough to slow the hot path, confirming the hypothesis's own refute clause — store-elimination only pays when the store actually survives to machine code, and this one did not. The link-surgery path needs an algorithmic reduction in detach/relink ops, not removal of writes the compiler had already dropped.
