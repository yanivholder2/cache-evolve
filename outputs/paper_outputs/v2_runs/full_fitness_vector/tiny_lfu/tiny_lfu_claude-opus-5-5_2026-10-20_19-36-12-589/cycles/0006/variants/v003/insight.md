# Variant v003 Insight

## Helped

- per-op-time: The gain comes almost entirely from folding in v001's fused `increment_hash`/`estimate_hash`, not from the window. `get` pays for one `increment_hash` on every call. That call used to run two loops over the four rows with bounds-checked indexing, and is now four `get_unchecked` loads plus branch-free stores, all in registers. `put`'s tail latency also dropped, even though the full-cache miss path now calls `key_hash` twice (candidate and victim) instead of once. That path also no longer clones the victim key before `map.remove`. And `relink` returns early when the node is already at its list's MRU end, so repeated hits on the hottest key skip the unlink/push. The parent's put p99 spike reversed even though the overflow path does more hashing. That supports the parent insight's suspicion that the spike came from codegen or drift, not from the miss/put fix.

## Hurt

- hit-rate: The hypothesis failed: the 10% window lowered hit rate on workload-a instead of raising it. Every new key now enters the window unconditionally, so a tenth of capacity always holds unfiltered arrivals, most of them one-hit keys. The parent's strict duel would have rejected those keys at the door. The frequency-guarded main region shrinks by the same amount. Bursts the window does catch get re-referenced and relinked within the window. They still have to win a duel against the main LRU head once they age out, and ties go to the resident. Here, the recency they buy is worth less than the frequent keys pushed out of the smaller main region.

## Takeaway

On this LIRS workload-a trace, a 10% LRU window sitting in front of TinyLFU costs more than it earns. The trace rewards strict frequency filtering at the door. A window large enough to hold bursts mainly gives cache space to one-hit arrivals, so the "untested 5–20% window" question now leans toward smaller windows here, not larger ones. The region-bit encoding kept `Node<u64,u64>` at 24 bytes; only the fixed per-cache struct grew, from the extra head/tail/len/cap fields. The fused sketch is a large, behaviour-neutral latency win, worth carrying forward independently of the window.
