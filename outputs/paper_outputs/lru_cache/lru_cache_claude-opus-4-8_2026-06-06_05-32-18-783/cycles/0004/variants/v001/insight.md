# Variant v001 Insight

## Helped

- metadata-size: Re-encoding the two `Option<usize>` recency links as bare `u32` indices with a `NIL` sentinel reclaims 24 of the 32 link bytes, and packing the `protected` segment flag into the high bit of `prev` (via `PROT_MASK`) avoids a separate `bool`+padding word — together they land `Node<u64,u64>` squarely in the 24-byte class rather than stopping at 32.
- per-op-time: The half-size node packs more of the arena into each cache line, and routing every link touch through the `slot`/`link`/`*_link` helpers keeps traversal a single zero-extend with no `as usize` sprinkled through the bodies, so both get and put got faster instead of paying a casting tax for the narrower representation.

## Hurt

- (none) — hit-rate held at the parent's record because the SLRU geometry was preserved byte-for-byte: `prot_cap` soft bound, probation-first victim selection, the protected-LRU fallback, and demote-on-overflow all operate identically on the new `u32` links.

## Takeaway

The node footprint was dominated by two 16-byte no-niche `Option<usize>` links, not by anything policy-bearing, so narrowing them to `u32`+sentinel and stealing the `prev` high bit for the segment flag was a pure representation change that crossed two size classes (56 to 24 bytes) without disturbing eviction behavior. Packing the flag rather than keeping a separate `bool` is precisely the move that turns a 32-byte win into a 24-byte one, and the tighter layout repaid itself in lower latency.
