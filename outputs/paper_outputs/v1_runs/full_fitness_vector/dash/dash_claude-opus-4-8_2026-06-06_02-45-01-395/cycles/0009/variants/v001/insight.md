# Variant v001 Insight

## Helped

- per-op-time (put): Gating the retry on `!is_full()` makes the non-full branch a single `put_at(Slot::Target)` that can never evict, so the parent's cascade second `put_at` is structurally gone — the double-hop that cycle 0007 pinned as the put-p99 bound was removed and put-p99 fell as predicted.

## Hurt

- per-op-time (get): Routing full-target foreign residents straight to `Slot::Stash` instead of evicting-and-cascading inflates stash occupancy, so get lookups that scan the stash pay a longer chain — both get-p50 and get-p99 climbed, and since get-p50 is the per-op-time summary the dimension regressed overall.
- hit-rate: The `is_full()` route-to-stash branch still concedes exactly the rescued hits whose true target was full, so hit-rate slipped below the parent rather than holding at the parent's value — the guard recovers more than a full collapse would, but not all of the parent's rescue.

## Takeaway

The guarded retry confirmed the cascade `put_at` was the put-p99 bound — gating it on a non-full target removed that tail cleanly. But the "cheaper" stash redirect is not free: diverting full-target items into the stash lengthens the stash that the get path scans, trading the put tail for a worse get-p50/p99 (a net per-op-time regression on the summary op) while still leaking the full-target rescued hits. The parent's double-hop wasn't pure waste — it was keeping the stash short at the get path's expense.
