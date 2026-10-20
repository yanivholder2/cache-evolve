# Variant v003 Insight

## Helped

- per-op-time: The put tail latency dropped, plausibly because the repacked per-segment fields mean fewer bytes are written per insert; this link is inferred, since the overlay source could not be inspected in this session and the measurement is a single run.

## Hurt

- per-op-time: The get tail latency rose, plausibly because reading the packed segment fields now needs extra shift and mask work on the lookup path; this is also inferred from the hypothesis rather than confirmed in the source.

## Takeaway

The intended change, shrinking `DashSegment` from 24 to 16 bytes by reordering fields and packing small counters, did not land: the reported segment size and the overall metadata-size are unchanged from the parent, so the layout claim remains untested. Hit rate is identical to the parent, which is consistent with eviction order being untouched. The only movement is mixed per-op tail latency with no change to the summary, so this variant gives no clear evidence for the packing change and the per-op effect should be re-measured in isolation before being credited.
