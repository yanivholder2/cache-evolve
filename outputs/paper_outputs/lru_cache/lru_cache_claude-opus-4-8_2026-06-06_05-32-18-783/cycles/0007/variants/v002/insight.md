# Variant v002 Insight

## Helped

- (none) — the stricter two-hit admission gate did not lift hit-rate above the parent; it fell below it.

## Hurt

- hit-rate: Requiring a second probationary hit before promotion (warm-then-promote in `get`'s probationary arm) makes entries dwell longer in probation, so entries with short reuse distances get evicted as the probationary LRU before they earn their second hit — the protected segment is starved of exactly the recurring keys it should shield.
- per-op-time: The added per-hit bookkeeping — `set_warm` plus the extra warm-flag branch on every probationary `get` — nudged get p50 up, since the common first-hit case now does link surgery without the promotion that used to pay for it.

## Takeaway

For this workload, gating promotion behind two probationary hits is too strict: short-reuse-distance keys drain out of probation before the second hit lands, so the one-hit-wonder filter also discards genuine repeaters and hit-rate drops below the single-hit-promotion parent. The free-bit mechanism itself worked as intended — parking the warm flag in the high bit of `next` via the masked `next_link`/`set_next_link` helpers kept `Node` at 24 bytes — so the cost was admission policy, not layout.
