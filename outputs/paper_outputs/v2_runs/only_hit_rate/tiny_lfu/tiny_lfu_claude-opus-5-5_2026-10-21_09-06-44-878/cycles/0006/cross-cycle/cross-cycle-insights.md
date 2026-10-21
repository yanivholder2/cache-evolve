## Recurring wins

- Estimate quality has been the only lever with a clearly large hit-rate gain. Widening the sketch to remove collision overcount (cycle 3) beat every contest, sampling and structural tweak. Making counts honest by deduplicating the get-miss-then-put double increment (cycle 4) added a smaller gain on top.
- Lifetime (non-aging) frequency is the right signal for workload-a. Every lineage that kept it intact did at least as well as the lineages that added freshness.
- Strict `>` with keep-victim-on-tie admission has held up. Every attempt to loosen it lost hit-rate.
- Frequency-aware victim selection (a small sample of the oldest entries, picking the one with the minimum estimate) gave small but repeated gains. It now acts as a soft protected segment.
- Changes that preserve semantics (fast single hash, hash-cached fast path, narrower conservative-update sketch on the non-SLRU parent) reliably match the parent's hit-rate exactly. They are safe to stack once their cost axes are scored.

## Recurring failures

- Aging or decay failed in every form tried: short-period with conservative update, long-period alone, and very-long-period on a clean sketch. Freshness keeps losing to lifetime counts on this trace. Early "stale-high" blocking came from collision noise, not from old popularity.
- Admission windows lost every time: a 1% window on a noisy sketch, an adaptive hill-climbed window, and a 1% window in front of SLRU on clean estimates. Few keys come back within a short recency trial, and the slots taken from the main region cost more hits than the window recovers.
- Loosening the contest failed: tie-admission swapped one cold key for another. Widening the victim sample failed too, because on a non-aging sketch the lowest estimate marks the youngest resident, not the coldest.
- Once the sketch was clean, tuning its accuracy stopped helping: conservative update on a wide sketch did nothing, because width and conservative update fix the same error.
- Tweaks inside SLRU (demotion policy, window, narrow sketch) only rearrange entries. The resident structure is already over-defended.
- Systemic: for six cycles running, per-op-time and metadata-size have not been scored. Memory and latency variants can therefore only tie or lose, and their real benefit stays unverified. This is now the main blocker on progress.

## Open questions

- Will the evaluator score per-op-time and metadata-size? If it does, which stack dominates once costs are priced: SLRU, the narrow CU sketch, the hash-cached fast path, or all three combined? What do SLRU's per-hit list work and per-promotion estimates cost?
- Inside SLRU, could a recency tie-break between equal estimates let the sketch drop below 8× width without flipping admission or victim decisions?
- Is the flat hit-rate on workload-a (unchanged across cycles 5–6 over policy, structure and sketch levers) simply this trace's ceiling for TinyLFU-family policies? Would a different workload change any of the conclusions about aging or windows?

## Per-policy notes

- Count-min sketch: width matters most. In plain TinyLFU it is over-provisioned (half width plus u16 counters plus conservative update was neutral). In SLRU with probation-only sampling, 8× looks like the floor, because leftover collision error flips some decisions. Keep dedup of the get-then-put increment.
- TinyLFU admission contest: keep strict `>` with keep-victim-on-tie. A first-time newcomer still ties a resident that was accessed once and loses, but cycle 6 showed these lost ties are not where the remaining misses are.
- W-TinyLFU (window + main): consistently negative on workload-a, whether fixed or adaptive. A naive climber drifts toward an LRU-like window, which this LIRS-style trace punishes.
- SLRU probation/protected: a marginal win that largely duplicates min-estimate victim sampling. Demotion by frequency held hit-rate unchanged but adds unpriced estimate cost.
- Fast-path shortcuts (early stop or early reject at estimate ≤ 1) are valid only without aging. Any decay or reset proposal must remove or re-derive them.
