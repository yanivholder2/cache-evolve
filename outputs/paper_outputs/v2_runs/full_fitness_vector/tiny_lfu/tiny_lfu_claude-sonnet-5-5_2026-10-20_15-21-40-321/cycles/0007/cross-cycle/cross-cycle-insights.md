## Recurring wins

- Footprint and locality reductions (compact Node, link layout, packed segment bit) are the only lever that has moved headline dimensions. Padding and alignment set a floor, so further gains need a layout change that crosses an alignment boundary.
- Cheaper hashing is the clearest speed lever, both in the sketch (cycles 1-2) and in the map hasher (cycle 4). Each win left hit-rate flat.
- Changes that make an existing component cheaper without altering admission semantics keep hit-rate flat and sometimes help speed. Changes that add mechanism rarely do.

## Recurring failures

- Hit-rate levers have all failed on workload-a. These were aging (including a long period), a small window, a doorkeeper, a wider sketch, conservative update, a wider victim sample, tie-admitting comparison and a segmented main LRU. Frequencies are stable, often tied, and the baseline strict-greater admission already fits the trace.
- Sketch-internal tweaks are exhausted: width, counter size, hash derivation, early-exit and conservative update. The single-hash speedup was lineage-specific and did not stack on stronger parents.
- Adding state or hot-path work (doorkeeper bitset, extra fields, segment lookup) raises tail latency and grows the struct.
- Micro-optimizations such as pre-sizing, forced inlining, branch hints and removing a cached Node field gave nothing. Allocation and call overhead are not bottlenecks.
- Small per-op latency deltas have repeatedly proved to be noise, including improvements with no plausible code cause. Many "wins" are unverified.
- The metadata headline tracks Node size only, so sketch heap changes are invisible there. Many insights were also written without a source diff, so attributions are inferred.

## Open questions

- Where do `get` and `put` actually spend time? No profile exists, and the non-sketch hot path (map lookup, relinking, bounds checks) is untested.
- Does the compact Node alone reproduce the speed gain on the best incumbent, without segmentation, and does it survive repeat runs?
- Is any hit-rate lever left on workload-a, for example window sizing, a different admission decision, or a drifting trace that would justify decay?
- Do the best pieces stack (cheap map hasher with MRU skip, compact links, compact sketch) without losing hit-rate?
- How much of the observed latency variance is noise, and what repeat-run protocol is needed?

## Per-policy notes

- TinyLFU (sketch admission with LRU main): the baseline design is near a local optimum for hit-rate on workload-a. The remaining headroom is per-op cost and footprint.
- Best speed and size lineage: compact u32-link layout, then cheap map hasher with MRU short-circuit. Merge compact Node packing onto it before trying anything else.
- Sketch: a single hash with u8 counters and conservative update is the established cheap form. Nibble packing trades hit-rate for footprint. Width and accuracy changes are saturated.
- Window and segmented LRU variants: no hit-rate gain so far. Any retry needs a rationale tied to a specific trace property.
