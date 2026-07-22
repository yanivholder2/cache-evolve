# Variant v001 Insight

## Helped

- hit-rate: Grafting v002's scan-resistant `ClassicLRU` — bumping `lfu_counter` on every hit and evicting the first `lfu_counter == 0` (one-hit-wonder) entry — protects genuinely reused items from scan bursts, lifting hit rate well above the parent's plain move-to-back/evict-front LRU.

## Hurt

- per-op-time: The new `eviction_candidate_index` does a linear `position(|it| it.lfu_counter == 0)` scan instead of returning a constant 0, nudging put p99 up; the summary held, so this is a small tail cost of the reuse-aware scan, not a systemic regression.

## Takeaway

The scan-resistance logic rides entirely on the `lfu_counter` field that v003's slimmed 24-byte `Item` already kept, so v002's hit-rate win and v003's metadata win compose cleanly with no layout conflict — the reuse flag needs no extra bytes. Confirms the two cycle-0001 winners were touching disjoint mechanisms and stack additively.
