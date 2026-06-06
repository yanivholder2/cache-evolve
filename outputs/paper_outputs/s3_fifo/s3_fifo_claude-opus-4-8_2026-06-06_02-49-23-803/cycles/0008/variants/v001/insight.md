# Variant v001 Insight

## Helped

- per-op-time: put p50 and p99 fell sharply because the linear `ghost.iter().any(..)` membership test plus the `ghost.retain(..)` rebuild — both O(ghost_len) over a queue sized ~main_size/2 — were replaced by a single `ghost_set.contains`/`remove`, so the last linear walk on the hot `put` path is now O(1).

## Hurt

- metadata-size: the scored `Entry<K,V>` stays 24 bytes, but the unscored `S3Fifo` container grew because the new `ghost_set: HashSet<K>` field means each ghosted key is now stored twice — once in the order `VecDeque` (kept only for FIFO order, with lazy tombstones) and once in the membership set.

## Takeaway

Eliminating the scan — the same structural move v001 used on `get` — is what crushes put latency here: trading the ghost queue's linear membership/removal for an O(1) set, with lazy deletion keeping the VecDeque purely for eviction order, drops put p50/p99 by an order of magnitude while holding hit-rate bit-identical at 0.7312. The cost is the duplicated per-ghost-key storage in the container, and the get path is byte-identical so its p99 wobble is measurement noise, not a regression from this change.
