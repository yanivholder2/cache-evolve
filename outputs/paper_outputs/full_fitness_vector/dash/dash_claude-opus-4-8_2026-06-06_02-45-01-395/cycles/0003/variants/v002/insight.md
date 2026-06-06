# Variant v002 Insight

## Helped

- hit-rate: Replacing the positional `promotion_candidate_index()` test with a `PROMOTE_AT == 2` frequency gate means a key only enters the protected probing tier after two proven target hits, so genuine repeat warmth — not positional luck — steers promotions, nudging hits up.

## Hurt

- per-op-time: Every target hit now does a read-modify-write of `lfu_counter` (`saturating_add` plus a `get_items_mut` write) on the hot `get` path before the gate decides, raising get and put p50.

## Takeaway

Spending the free `u8 lfu_counter` (the `Item` is 24 bytes either way, so metadata-size held flat) as a "promote after N proven hits" gate confirms frequency beats positional luck for steering keys into probing — but only barely on this trace, and the per-hit counter bump is not free in time. A frequency signal buys a sliver of hit-rate at a measurable latency cost, so the gate's value hinges on whether the hit-rate gain is worth the hot-path write.
