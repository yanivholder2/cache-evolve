## Recurring wins

- Cutting repeated hashing cost (sketch hashing early on, then the map hasher) was the only speed lever that paid off clearly. Gains showed in the put tail, with hit-rate unchanged.
- Changes that left admission (strict greater-than, LRU-head victim, conservative sketch) untouched kept hit-rate flat. That makes them safe vehicles for speed work.
- Sketch simplification on the baseline lineage (single hash, small counters) helped speed and footprint early. It did not stack on later lineages.

## Recurring failures

- Hit-rate levers have all failed on workload-a: aging (including long periods), small and smaller windows, wider and narrower sketches, doorkeeper, conservative update, victim sampling, tie-admitting, and segmented main LRU. Hit-rate looks plateaued for this design.
- Footprint cuts (u32 links, compact Node, halved sketch, field removal) rarely turned into speed. The working set is small, so locality gains are absent and conversion or unpacking overhead is paid on every op. Padding also absorbed some Node savings.
- Added state or hot-path work (doorkeeper bitset, extra fields, inline hints) raised tail latency and grew the struct.
- Blind hot-path micro-tweaks (pre-sizing, inlining, branch hints, early-exit) were flat or worse. Allocation and call overhead are not the bottleneck.
- Small latency deltas are often noise. Several "gains" came from untouched code paths, and the Winner label mostly reflects the selection rule.
- The metadata headline tracks Node size only, so sketch heap changes are invisible there.
- Many insights were written without a source diff, so attributions are partly inferred.

## Open questions

- Where do get and put actually spend time (map lookup, relinking, sketch estimate)? No profile exists.
- Is any hit-rate lever left on workload-a, or is it at its ceiling? A materially different mechanism, such as an adaptive window, is untested.
- Are the incumbents at the floor for this architecture, so that further gains need a different data structure or map?
- Does an earlier speed gain (cycle 0007's segmented variant, cycle 0004's hasher) survive repeat runs?
- Is there any trace with drift or larger frequency gaps that would justify aging or revisit the window?

## Per-policy notes

- TinyLFU (sketch plus window plus LRU main) is the only policy in the ledger. Its sketch is saturated for accuracy, and its admission rule is well matched to workload-a.
- Window: a small window is fine, and shrinking it further hurt put and hit-rate slightly. Do not tune it as a standalone lever.
- Sketch: low-precision, single-row variants trade hit-rate for footprint. Width, update rule and increment cost are exhausted.
- Segmented main LRU adds no hit-rate over sketch admission. Its apparent speed gain was not reproduced by the compact Node alone.
- Next cycles should obtain a profile first, name the cost each hypothesis targets, and use repeat runs. Avoid building on cycle 0008 v002 without auditing its diff, since it shows unexplained size growth.
