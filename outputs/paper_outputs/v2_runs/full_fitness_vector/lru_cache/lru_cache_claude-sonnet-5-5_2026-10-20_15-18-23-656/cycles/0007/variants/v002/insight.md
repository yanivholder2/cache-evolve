# Variant v002 Insight

## Helped

- hit-rate: Cold, first-time keys now enter at the LRU end of probation unless the ghost table remembers them, so one-touch scan keys become the next victim instead of displacing keys that have already been re-referenced.

## Hurt

- per-op-time: `put` is slower, mainly at the tail. Each eviction now hashes the victim key and writes the ghost table, and each miss does a ghost lookup and clear. `get` is unchanged.
- metadata-size: The `LruCache` struct grew by the `ghost: Vec<u32>` field, and the table itself adds heap memory of at least capacity entries. The per-`Node` size is unchanged, so the summary metric doesn't show this cost.

## Takeaway

A ghost fingerprint filter combined with LRU-end insertion for unseen keys gives a real scan-resistance gain on top of the parent's probation/protected split. It pays for that with an extra hash and a table access on every eviction and miss, and with side memory that the Node-size summary hides. The trade looks worthwhile only if hit-rate matters more than put-tail latency and the off-Node memory.
