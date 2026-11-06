# Variant v002 Insight

## Helped

- per-op-time (get p99): The get tail dropped sharply. That matches the one thing the hypothesis did remove: on a protected hit, the `unlink` + `push_back` relink that wrote to the prev, next and `prot_tail` nodes (often cold at 131K entries) is gone. `on_hit` now does one conditional OR into `prev` on a node the lookup has already loaded. Treat this cautiously: the parent's get p99 rise came from no code change, so get p99 is known to swing between runs. Part of this improvement may be that swing reversing, not the CLOCK change.
- hit-rate (web_0 @ 131072): This was the one trace/size that improved. With a large protected segment, a second-chance sweep that rotates referenced heads past demotion seems to keep web_0's re-referenced working set as well as strict LRU, or slightly better.

## Hurt

- per-op-time (get p50): Get p50 rose a lot instead of falling, so the central prediction failed. The saved relink on protected hits did not show up at the median. Meanwhile each probation→protected promotion now runs `demote_one`, which can rotate up to `MAX_SWEEP` cold protected heads, each with an `unlink` + `push_back` that touches the head, its successor and the tail. Under CLOCK, hot entries stop moving, drift to the LRU end and stay referenced, so sweeps routinely rotate several nodes instead of exiting at the first one.
- per-op-time (put p50/p99): Put got slower even though the eviction path is effectively unchanged. The only edits there are that `push_back` now writes `prev` directly and that occupied-key puts go through the now-cheaper `on_hit`. No new work on the put path explains a rise of this size. Get p50 and put p50 rose by similar absolute amounts, which points at least partly to a run-environment shift, not only to the sweep.
- hit-rate (web_0 @ 16384, hm_0 @ 16384, hm_0 @ 131072): Hit rate fell slightly on these three. A single reference bit only records "hit at least once since the last rotation", not recency order. The forced demotion after 8 rotations can also push out a referenced entry. Both cost the most where the protected set turns over fast (the small capacity) or where hm_0's recency order matters.

## Takeaway

Replacing protected-hit relinks with a CLOCK reference bit did not make get p50 cheaper. Only get p99 improved, and only if that isn't run-to-run noise. The list work moved into a bounded sweep on promotion, which seems to cost more per call than it saves on the protected hits. Hit rate also drifted down on three of four trace/size pairs. Before building on this, re-run parent and child together (put moved with no code cause), and count sweep rotations per promotion to see whether `demote_one` is where the median cost went.
