# Variant v002 Insight

## Helped

- hit-rate: Hit rate rose clearly over the parent and clears the bar the hypothesis set. Fewer items leave the cache because two levers draw on different spare capacity. First, a stash overflow spills into the neighbour bucket `s1` before anything is evicted. Second, when both `s0` and `s1` are full, the victim is re-homed into a free slot in its own target or probing bucket. Those free slots are the holes that promotion `remove_at` leaves behind. Both levers landed in one step over v001, so this variant alone cannot say how much of the gain comes from each.

## Hurt

- per-op-time (get p50/p99): Median and tail `get` both got clearly slower, more than run-to-run noise. `promotion_get` keeps its stash-first order, and `find_in_stash` now scans up to two stash buckets (`s0`, then `s1`) before it looks at target or probing. So every common target/probing hit and every miss pays an extra bucket scan. Rescue is not the cause, because it adds no work on the lookup path.
- per-op-time (put p50/p99): `put` also slowed. Every insert can now check `s1` for fullness. When both stash choices overflow, it also computes the victim's target/probing index and checks those buckets for fullness, and it may run a `put_at` into a normal bucket. The same wrapper runs on both demotion inserts inside promotion cascades, which also lengthens the slow `get` promotions.

(metadata-size did not change: no fields were added, and `Item` stays at 40 bytes.)

## Takeaway

Two-choice stash plus overflow rescue does stack into a real hit-rate gain, and rescue costs nothing on the lookup path because it never evicts. The latency bill comes mostly from the stash-first `get` order, which now scans two stash buckets before reaching the normal buckets, where most hits live. The natural follow-up is to keep this placement logic but check target/probing before the two stash buckets in `promotion_get`. Rescue keeps the key-uniqueness invariant, so that reorder may not change behaviour, though this variant doesn't show that. If it holds, the reorder could win back most of the `get` cost without giving up the hit-rate.
