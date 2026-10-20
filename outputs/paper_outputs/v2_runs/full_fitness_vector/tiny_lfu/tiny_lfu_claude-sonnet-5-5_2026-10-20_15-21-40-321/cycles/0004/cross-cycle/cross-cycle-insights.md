## Recurring wins

- Cutting per-op cost in the existing machinery pays off: single hash, cheaper index derivation, cheap map hasher. These keep helping speed and tail latency with hit-rate unchanged.
- Footprint reductions that leave counters, conservative update and admission untouched (u8 counters, u32 links) keep hit-rate flat. The nibble-packed sketch shrinks the footprint further but costs hit-rate.
- The map hasher was the largest remaining latency cost, and swapping it recovered the time lost to the compact layout while keeping the size gain.
- Gains that show in headline dimensions (latency, Node size) are the ones worth pursuing.

## Recurring failures

- Aging or halving, at any period, failed twice. workload-a has stable frequencies, so decay discards useful history.
- A window, a doorkeeper, a wider sketch, a wider victim sample and tie-admitting `>=` all failed to lift hit-rate. Some lowered it. Sketch accuracy looks saturated, and strict greater-than admission with the LRU head as victim suits this trace.
- Extra state or hot-path work (bitsets, segment lookups, cached fields) raises tail latency and grows the struct without a matching benefit.
- Shrinking sketch precision too far (4-bit, one row, tight sizing) trades hit-rate for footprint.
- Small latency deltas are often noise or code-layout effects. Several unexplained moves have been seen, so single-run deltas are unreliable.
- The headline metadata number tracks Node size only. Sketch heap changes, both savings and costs, are invisible in it.
- Several attributions were inferred without a source diff, so they are not confirmed.

## Open questions

- Is any hit-rate lever left on workload-a? Admission, victim choice, tie rules, aging, windows and sketch accuracy are all exhausted, so a different mechanism is needed.
- Do the compact nibble sketch, the cheap hashing and the cheap map hasher stack without losing hit-rate? How much accuracy does the nibble layout need back, and which part of its loss comes from the cap, the single row or the sizing?
- Which part of the map-hasher change accounts for its gain: the hasher or the MRU skip? Does it hold under repeat measurement?
- Is there a trace with drift or large frequency gaps where aging, a window or the u8 cap would matter?
- How much latency variance is noise, and is repeated measurement needed?

## Per-policy notes

- TinyLFU (the only policy so far): the baseline's design (never-decaying counters, strict-greater admission, LRU-head victim) is hard to beat on hit-rate on workload-a. Progress has come from making it cheaper and smaller.
- Current best lineage: single-hash conservative sketch, then the compact u32 link layout, then the cheap map hasher with the MRU short-circuit. The nibble sketch and the cached-hash speedup are not yet combined with it.
