# Cycle 0007 Insight

## Cohort outcome

This cycle explored three independent axes — best-of-2 victim selection on the amortised gate path, shrinking sketch counters from u32 to u16, and doubling the sketch width factor. All three variants were accepted as winners despite mixed results: v001 was latency-neutral but gained nothing on hit rate; v002 halved the sketch heap but paid a p99 latency increase; v003 raised hit rate but doubled p99 tail latency.

## Patterns

- **Best-of-2 victim selection at the LRU cold end is a closed axis.** v001 confirmed that both the LRU head and its immediate neighbour are equally cold relative to the working set — the admission gate comparison against the incoming key is the operative filter, not which cold candidate is chosen. The hypothesis is refuted even under the optimal implementation that eliminates the redundant estimate call.
- **Sketch width is a genuine hit-rate lever; counter width is not a free-latency lever.** v003 showed that halving expected bucket collision rate (4× → 8× width) does lift hit rate, confirming that collision noise was suppressing admission of genuinely hot keys. v002 showed that halving the heap via u16 counters does not automatically improve latency — the per-access zero-extension overhead outweighs the cache-footprint reduction at these sketch sizes.
- **A compound u16 + 8× width sketch is an untested configuration at the same heap size as today's baseline.** The current 4× u32 sketch occupies 64 KB. An 8× u16 sketch would also occupy 64 KB while halving collision rate. Neither this cycle nor any prior cycle has evaluated this combination, and it avoids the competing penalties that broke v002 and v003 individually.
- **p99 tail latency is the binding constraint, not p50.** Both v002 and v003 left p50 largely unaffected while moving p99 significantly — reinforcing that the sketch is on a sporadic, high-cost code path rather than the hot median path.

## Open questions

- Does an 8× u16 sketch (same 64 KB heap as the u32 4× baseline) achieve v003's hit-rate gain while avoiding v002's per-access widening cost, because a narrower element type at doubled width keeps the same total bytes in cache?
- Are structural policy parameters — window/protected segment ratio, probationary-to-protected promotion threshold — genuinely unexplored levers, or do they interact with sketch accuracy in ways that require the sketch to be tuned first?

## Continuation

The natural next hypothesis is the u16 + 8× width compound: the combined sketch heap matches the pre-existing 64 KB baseline, so the cache-footprint penalty of 8× width is neutralised while the lower collision rate of v003 and the narrower counter access of v002 may each contribute independently. This is the highest-priority untested thread. The best-of-2 victim selection axis is definitively closed and should not be revisited. If the u16 + 8× compound exhausts quickly, shift to structural policy parameters (window ratio, segment balance, promotion threshold) — none of the prior seven cycles have touched these levers.
