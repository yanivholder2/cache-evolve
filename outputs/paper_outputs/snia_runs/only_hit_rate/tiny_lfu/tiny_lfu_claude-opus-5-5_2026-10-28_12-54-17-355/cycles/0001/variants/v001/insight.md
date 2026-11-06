# Variant v001 Insight

## Helped

- hit-rate: The baseline sent every first-time key into the admission contest against the main LRU. A new key had never been seen before, so it almost always lost and could not hit on a quick re-reference. The 1% LRU window now admits newcomers unconditionally, and they only face the sketch contest when they age out of the window. This recovers short-reuse hits on both web_0 and hm_0. The gain is largest at the smaller cache size, where the baseline's reject-on-full rule fired most often.

## Hurt

## Takeaway

On these MSR block traces, the baseline's main weakness was its admission policy, not its eviction order. A key needed sketch frequency before it could ever become resident, so short-term recency was lost. A small window that buffers newcomers fixes that without removing the frequency filter for the main region. The gain gets smaller as capacity grows, which suggests that window size relative to the reuse distance of recent writes is the next knob to tune. This vector measured only hit-rate. The cost of the new per-node `region` tag and the second list's bookkeeping (metadata-size, per-op-time) is still unmeasured.
