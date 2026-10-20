# Variant v003 Insight

## Helped

- metadata-size: unchanged. The hash-parity victim choice reuses the already-computed hash, so no per-entry state was added.

## Hurt

- hit-rate: Choosing between the last two slots by a hash bit evicts the second-oldest entry about half the time. That is an arbitrary choice, not a better one. It sometimes throws out a more recently useful entry and keeps a colder one, so it does no better than strict LRU-end eviction on this trace.
- per-op-time: put and get both got slower. The parity branch and the swap that fills the freed slot add work on the miss path. The branch is also unpredictable, because it depends on key-hash bits. The get slowdown is more likely measurement noise or layout effects, since get was not changed.

## Takeaway

A stateless, randomized second chance for the LRU-end entry gives no signal about reuse, so it cannot imitate a reference bit or a ghost filter. It only adds noise to eviction and an unpredictable branch to the put path. Any scan-resistance gain has to come from information about the entry, such as recency or frequency, and key-hash bits do not carry that.
