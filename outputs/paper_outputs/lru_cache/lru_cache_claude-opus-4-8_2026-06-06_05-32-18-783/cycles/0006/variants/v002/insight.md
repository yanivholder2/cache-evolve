# Variant v002 Insight

## Helped

- (none) — no dimension improved. hit-rate held because `move_to_mru` performs the same detach-then-tail-append surgeries as the pair it replaced, preserving final ordering byte-for-byte; metadata-size held because `Node` stayed 24 bytes (the fusion changed only code, not layout).

## Hurt

- per-op-time: Fusing `unlink`+`push_back` into the `#[inline] move_to_mru` regressed both get and put at p50 and p99. The "dead" `set_prev_link(NIL)`/`set_next_link(NIL)` stores it elided were writes to a node already hot in cache and immediately overwritten — the store buffer absorbs those nearly for free, so removing them bought nothing, while inlining the larger fused body into the hot get/put paths added code size and register pressure that cost more than the stores saved.

## Takeaway

Counting eliminated writes is the wrong cost model here: the two sentinel stores `move_to_mru` removes are store-buffer-absorbed writes to an already-resident cache line, so they were never the bottleneck, and folding the whole relink into an inlined helper on the hot paths traded a non-cost for real code-size/register pressure and made every refresh slower. Routing get/put refreshes through the separate `unlink`/`push_back` calls (as the parent did) is the faster shape despite the redundant NIL writes.
