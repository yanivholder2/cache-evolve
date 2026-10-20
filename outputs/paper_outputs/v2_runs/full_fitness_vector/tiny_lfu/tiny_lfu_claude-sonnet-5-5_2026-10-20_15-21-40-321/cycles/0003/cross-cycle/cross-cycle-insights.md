## Recurring wins

- Making the sketch cheaper (fewer hashes, single mix, bit-sliced indexes, smaller counters) is the most reliable gain. It cuts per-op time, especially the put tail, and shrinks the sketch.
- Changes that leave counters, conservative update and admission untouched keep hit-rate flat, so they are safe speed wins.
- Layout shrinks (u32 indices, compact sketch) reduce footprint without hurting hit-rate.

## Recurring failures

- Aging or decay of any period fails on workload-a. Its frequencies are stable, so decay discards useful history. This has failed in several forms.
- Extra admission mechanism (small window, victim sampling, tie-admitting `>=`) adds cost and gives no hit-rate gain. Admitting ties actively lowers hit-rate because ties are common at low frequencies.
- Cutting sketch precision too far (4-bit counters, one row, tight sizing) buys speed and footprint but loses hit-rate.
- Extra hot-path state or conversions (cached state, NIL checks, index widening) tend to cost latency that outweighs the locality gain on a small working set.
- Small cache-struct additions grow Node size. The headline metadata number tracks Node size, not sketch heap, so sketch savings are invisible there.

## Open questions

- How can the nibble sketch recover hit-rate with the least added cost (second hash, larger table)? Which factor causes the loss: the 4-bit cap, one row, or the sizing?
- Do the single-hash speedup, the compact sketch and the u32 layout combine, or do they offset each other?
- Can the u32 layout's latency cost be recovered (fewer conversions, unchecked indexing), or is it noise?
- Is there a new signal that separates tied frequencies without hot-path work, such as a doorkeeper?
- How large is latency noise? Small time deltas need repeated or larger runs before they are trusted.
- Which parts of the u8/single-hash/conservative-update rewrite account for its gains?
- Is there any drifting trace that would justify decay or a larger window?

## Per-policy notes

- TinyLFU (strict greater-than admission, LRU head as victim, never-decaying counters): this baseline fits workload-a well. Gains come from sketch cost and accuracy, not from changes to admission comparison or victim choice.
- The rewritten single-hash u8 conservative-update sketch is the established base. The single-mix hash variant is the preferred speed parent because it has no hit-rate cost.
- The compact u32-index layout is the footprint parent, with a latency cost still to win back.
- Several insights were written without source access, so attributions of gains to specific code changes are inferred.
