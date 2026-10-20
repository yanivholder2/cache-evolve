# Variant v003 Insight

## Helped

- metadata-size (Node<u64,u64>): Packing the region flag into the top bit of `Node.next` (with `NIL` narrowed to 31 bits) gave the cache a window/main split with no extra bytes per entry. The headline summary stays flat. This also shows that cycle 0001's per-entry overhead was not a necessary cost of W-TinyLFU.

## Hurt

- hit-rate: The ~1% window (about 20 entries at size 2048) is too small to keep recency bursts on this trace. It also changes the admission order. Every new key now always displaces a window resident. The duel then judges that older window head instead of the new arrival, so some keys v001 would have admitted on arrival frequency get dropped, and some cold ones briefly displace warm window entries. The net result is slightly below v001, not the gain the hypothesis predicted.
- per-op-time (get): A hit-path relink now has to branch on `is_window` twice and pick from four head/tail fields. Every `next` write is a masked read-modify-write through `set_next`, and the window path also keeps `win_len` up to date. `get` runs `unlink`+`push_back` on every hit, so this list overhead shows up in both p50 and p99.
- per-op-time (put): v001 hashed the incoming key once and reused the estimate from `increment_hash`. On a full miss, `make_room` now re-hashes two stored keys, the window candidate and the main head. It also always admits the new key, so a slot rewrite happens even when v001 would have rejected the key with no writes. When the candidate wins, there are three list relinks: evict the victim, move the candidate to main, and push the new key.
- metadata-size (TinyLfuCache struct): The four list endpoints plus the `win_len` and `win_cap` counters make the cache struct larger. The per-node summary doesn't move.

## Takeaway

A minimal admission window can be built without per-entry cost, but on this trace a ~1% plain-LRU window doesn't recover hit rate. It also costs more per operation than v001's single-list strict duel, because of the masked links, list selection, and the extra key re-hashes in the duel. So the cross-cycle open question gets a negative answer for this design: the remaining hit-rate gap is not closed by adding a small recency window ahead of the duel. If future work keeps a window, it should cache each node's hash or estimate so the overflow duel doesn't re-hash keys, and it should test a larger window before calling the idea dead.
