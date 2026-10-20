# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Replacing the per-row seeded hash with one `DefaultHasher` pass plus a xor-and-multiply mix in `position_from` changes which counters each key lands in, so the sketch's collision pattern shifts and the estimates that drive the admission comparison move slightly downward.

## Takeaway

The change keeps SipHash once per key and derives all four row positions from that 64-bit value, so the sketch's counter layout and admission rule are untouched and only the mapping from key to counters differs. The fitness vector for this variant reports only hit-rate, so the intended per-op-time gain is unmeasured here; the one observed movement is a small hit-rate dip, which is the side effect the hypothesis itself flagged as acceptable only if it stays within noise.
