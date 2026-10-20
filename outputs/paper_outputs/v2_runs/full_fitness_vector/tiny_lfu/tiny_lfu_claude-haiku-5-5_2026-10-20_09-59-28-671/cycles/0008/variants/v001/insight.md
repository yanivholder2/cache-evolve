# Variant v001 Insight

## Helped

- hit-rate: Conservative update on the frequency sketch increments only the minimum-valued counters for a key, so colliding keys stop inflating each other's counts; this is the mechanism the hypothesis targeted, and it produced a small gain over the parent on workload-a at size 2048.

## Hurt

- per-op-time: The extra pass over the key's rows on every increment (read all rows, take the minimum, then conditionally write) adds work to both the get and put paths, and the slower p50 and p99 for both operations match the overhead the hypothesis predicted.

## Takeaway

Changing only how colliding counts accumulate, while keeping row indices and the minimum-across-rows estimate fixed, is a usable lever on the admission gate: it bought a small hit-rate gain at a measurable per-op cost. The metadata footprint did not move, which fits the change adding no struct fields. The source diff could not be inspected in this review (no file-listing or content-search access was available), so this reading rests on the hypothesis text and the fitness vectors alone; the gain is from a single trace and size and should be treated as provisional.
