# Cycle 0009 Insight

## Cohort outcome

All three variants attempted to recover hit-rate gains beyond the 87.5% protected quota without incurring a latency penalty: one probed an intermediate quota (capacity/12, ~91.67%), one batched demotion events at the known-costly 93.75% quota, and one combined a faster FNV-1a hasher with the 93.75% quota to try to offset demotion overhead. None won — every variant improved hit-rate but each still regressed per-op-time above the measurement floor.

## Patterns

- **The zero-latency ceiling for quota compression is below 91.67% protected.** Even the capacity/12 intermediate step — the smallest quota increase tested — pushed get p50 above the floor. The safe operating range for per-op-time does not extend past 87.5%, and the gap between the last cost-free point and the first regressing point is narrow enough that finer-grained probing is unlikely to find a free lunch.
- **Demotion batching is defeated by quota-induced graduation frequency.** v002's batch-2 scheme was designed to halve demotion event frequency, but the tighter probationary window simultaneously raised graduation rates — more items earn promotion per unit time — so the net demotion burden on the hot path was unchanged or worse. Amortisation cannot offset a mechanism that increases the event rate it is trying to spread.
- **FNV-1a delivers a genuine structural benefit but insufficient latency headroom.** v003 confirmed a real 16-byte struct-size reduction from replacing the seeded SipHash builder with a zero-sized FNV type, and hash computation cost did fall. However, the demotion churn at 93.75% quota is the dominant cost driver and is larger than the hashing savings — the hasher speedup is real but not wide enough to absorb the policy overhead.
- **The hit-rate vs. per-op-time trade-off at quota > 87.5% is now confirmed from three independent angles.** Intermediate quota (v001), event batching (v002), and hash speedup (v003) all failed to break the coupling. The trade-off appears structural to this policy design rather than fixable by tuning one mechanism.

## Open questions

- FNV-1a has only ever been tested paired with quota increases; its net effect at the confirmed clean 87.5% baseline is unknown. Could it lower per-op-time on the current best parent, potentially creating headroom to absorb a modest future quota increase?
- With the quota axis exhausted and latency-reduction mechanisms (batching, faster hashing) insufficient at higher quotas, are there node memory-layout or cache-line alignment changes that could reduce the demotion cost itself rather than trying to offset it?
- Is there a fundamentally different policy structure — such as segmented LRU with lazy promotion rather than eager demotion — that achieves a similar effective protected ratio without the per-operation enforcement overhead?

## Continuation

The next cycle should accept 87.5% (capacity/8 probationary) as the permanent quota ceiling under the current policy structure and stop probing higher values — three independent strategies have failed to break the latency coupling. Two directions worth pursuing: (1) apply FNV-1a to the clean 87.5% baseline alone, as a pure per-op-time optimization never tested without a quota change; if it lowers latency at the current best point, it creates a new floor that future quota experiments can target. (2) Pivot to a structural axis — node memory layout, cache-line packing, or a policy redesign that avoids eager per-operation demotion enforcement — rather than continuing to tune the existing protected/probationary split. Do not revisit demotion batching, admission filters, or quota values above 87.5%.
