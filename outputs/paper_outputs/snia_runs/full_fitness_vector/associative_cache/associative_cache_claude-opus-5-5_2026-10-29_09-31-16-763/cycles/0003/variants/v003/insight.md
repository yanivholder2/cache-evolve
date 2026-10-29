# Variant v003 Insight

## Helped

- per-op-time (put p99): The put tail improved a little. This is the only part of per-op-time that got better. The swap leaves `clock_sweep` and in-place replacement unchanged, so the gain most likely comes from the existing-key update branch. Keys that are updated often have been moved toward the front, so their `get_position` scan ends sooner. This is a guess: the eval artifacts do not break the tail down by branch.

## Hurt

- per-op-time (get p50 / summary): The swap made the typical get clearly slower, which is the opposite of what the hypothesis predicted. A hit at position ≥ 4 used to write one byte. It now runs a bounds-checked `items.swap` that rewrites two 40-byte items, usually on two different cache lines. The hit position is also halved only once per hit. The benchmark's key distribution evidently does not keep re-hitting the same few keys enough for the shorter scan to pay back those writes. Each swap also pushes the item at `position / 2`, which may itself be hot, back to the deeper slot. So hot keys trade places with each other instead of settling at the front.
- per-op-time (put p50): The typical put also got slower. The existing-key branch now pays for the same two-item swap. A put that misses still scans the whole bucket in `get_position`, so moving hot keys forward does nothing for it.
- hit-rate: Hit rate was basically flat, with a very slight drop on three of the four trace/size points. The swap keeps the set of reference bits but changes which slot each bit sits in relative to `hand`. A freshly referenced item can be moved to a slot the hand has already passed. Or a cold item can be moved into a slot the hand is about to check. That shifts CLOCK's eviction order a little without making it any smarter.

## Takeaway

Halving a hit's position in a single CLOCK bucket costs more than it saves. Turning CLOCK's write-one-bit hit path into a two-slot, two-cache-line swap gave back the get-latency advantage that made the parent attractive, and the shorter key scan did not make up for it. Hit rate stayed the same, so the parent remains better on every dimension that matters. If the goal is still a faster `get_position`, the reordering should cost about as little as the reference-bit store. Two options are a compact key or tag array scanned before the items, or swapping only when the hit is very deep (e.g. past the first cache lines). Reordering on every hit is the wrong approach.
