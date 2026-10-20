# Variant v001 Insight

## Helped

- hit-rate: The two-pass spare credit on a rescued protected head gives back a sliver of the protection that the parent's flag-only retry dropped, so the head survives a few demotion passes without the list rotation the parent removed. The gain is marginal, so most of the lost protection was not recovered.

## Hurt

- per-op-time: The put path p50 and p99 and the get p99 rose slightly over the parent. The only new work is in `demote_prot_head` and the rescue branch of `select_victim`, both on the miss path, so the added credit bit tests and updates account for at most part of the rise; the rest may be measurement noise from a single run. Get p50 held flat, which fits the unchanged hit path (`get` still only sets `REFERENCED`).

## Takeaway

Storing a 2-bit credit in the existing flags byte lets a rescued protected head survive two demotion passes with no relinks and no change to node size, which is a cheaper route than the parent's list surgery. The hit-rate result shows that bounded credit recovers only a small share of the MRU rotation's protection, so the rotation's effect depends on the head being moved, not just on a retained flag. Caveat: this reading is based on the overlay source, the hypothesis, and the two fitness vectors; the per-op deltas are from one run, so the tail-latency change is not yet established as a real regression.
