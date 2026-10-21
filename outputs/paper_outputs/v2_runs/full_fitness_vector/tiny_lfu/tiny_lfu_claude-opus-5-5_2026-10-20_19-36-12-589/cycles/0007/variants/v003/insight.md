# Variant v003 Insight

## Helped

(none: `metadata-size` is unchanged because `second_candidate` only reads the existing `refbits` word and adds no fields or per-node state.)

## Hurt

- hit-rate: Comparing the newcomer against the colder of `a` and `b` lowers the bar it has to beat, so the strict `>` duel lets in more marginal newcomers. Because `b` is replaced in place while the hand stays parked on `a`, the newly admitted entry sits in a clear-bit slot right next to the hand. It is likely to be the next `b` and gets evicted by the next marginal arrival before it can earn a ref bit. That turns the slots just past the hand into a churn zone for low-frequency keys.
- hit-rate: Under a tie or a warmer `b`, `a` still loses, but when `b` is colder the warm, unreferenced `a` survives and is re-dueled on every later miss. The extra frequency signal mostly protects entries whose sketch count is inflated by history, not entries with current reuse.
- per-op-time: `put` tail latency grew because a full miss now does a second `key_hash` plus `estimate_hash` and touches another `nodes` slot, so it can miss cache twice, on top of the unbounded CLOCK sweep inherited from v001. `get` is byte-for-byte unchanged, so its regression has no code-level cause. It most likely comes from benchmark noise or code-layout effects in the rebuilt binary, and shouldn't be read as a real cost of this change.

## Takeaway

A second, frequency-chosen CLOCK candidate gives the duel more information, but that information points the wrong way. Picking the minimum of two estimates makes admission more permissive, and leaving the hand on `a` after replacing `b` makes freshly admitted keys the next cheapest victims, so they churn each other. For a two-candidate scheme to pay off, it likely needs to keep the admission threshold tied to `a`, using `b` only to choose *which* slot to evict, or to move the hand past both candidates so new arrivals get a full revolution to earn a ref bit.
