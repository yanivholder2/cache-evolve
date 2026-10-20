# Variant v002 Insight

## Helped

- per-op-time (put): Capping the second-chance scan at two reprieves bounds a miss to at most three head inspections, so the put tail no longer grows with the number of referenced nodes at the head; put p99 fell from the parent's value.
- per-op-time (get): The hypothesis left `get()` unchanged, so this improvement is not a direct effect of the victim-search change; it may reflect layout effects from the node change or measurement noise, and should not be credited to the bounded scan.
- metadata-size: The node shrank below the 48-byte target the hypothesis predicted, so folding the referenced bit into the pointer layout did more than the hypothesis described; the mechanism is unverified because the source overlay could not be read.

## Hurt

- hit-rate: Hit rate dipped slightly relative to the parent, which is the cost the hypothesis predicted: once the reprieve budget runs out, the scan evicts referenced entries that an unbounded second-chance pass would have spared.

## Takeaway

Bounding the second-chance scan trades a small hit-rate loss for a predictable miss-path cost, and the put tail is the main evidence that the trade paid off. The metadata result is larger than the hypothesis expected, so the node-layout change should be checked against the overlay before it is treated as the cause; the source could not be inspected in this session, so attributions here are inferred from the hypothesis text and the fitness vectors.
