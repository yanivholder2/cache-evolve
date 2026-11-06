# Variant v001 Insight

## Helped

- per-op-time (get p50, the summary): `get`/`on_hit` are byte-identical to the parent, yet get p50 still dropped. The only release-build change is narrowing `ghost` from `Box<[u16]>` to `Box<[u8]>`, which halves the table at both sizes (64 KB to 32 KB at 131K, 32 KB to 16 KB at 16K). This fits the idea that the ghost's cache footprint is crowding out map and arena lines that `get` relies on. It is the third time a ghost-only byte cut has moved get, so it is unlikely to be pure noise.
- per-op-time (get p99, put p50/p99): put's `Vacant` arm reads and writes ghost slots, so a denser table means fewer cache misses on the lookup probe and on the victim-tag store. Put latency fell more sharply than get, which is consistent with put being the only path that touches the ghost directly. The tail latencies of both ops improved too, which also points to fewer cache misses.

## Hurt

- hit-rate (web_0/131K, hm_0/16K, hm_0/131K, summary): These three points slipped by the smallest step the metric shows. That matches 8-bit tags colliding much more often than 16-bit ones. A collision gives a never-evicted key a spurious second-chance flag, and the reprieve then keeps that key at the expense of a better victim. web_0/16K was unchanged. The new `spurious_reprieves_are_rare_with_8bit_tags` test bounds this cost at about 1/16 of genuine reprieves on a synthetic workload. On the real traces the cost shows up, but it is marginal.

## Takeaway

Narrowing the ghost tag from 16 to 8 bits stacks with the parent's capacity/4 length. Each byte removed from the ghost table pays off in latency for both get and put, even though no hot-path instruction changed. The cost is a small but consistent hit-rate loss from tag collisions. Because the hidden ghost footprint is clearly a latency lever, the next experiment could test whether the ghost is worth its bytes at all at 131K, where its reprieve window is shortest. One way: compare a 4-bit or zero-length ghost there against this one. `metadata-size` still does not count the ghost table, so this gain never shows up in that dimension.
