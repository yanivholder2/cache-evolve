# Variant v002 Insight

## Helped

- hit-rate: Every cell stayed within the hypothesis's ±0.0005 band of v003, and the four small moves all went up. hm_0 kept v003's gain at both sizes. So halving `PROBE_WINDOW` did not interfere with the bounded stash-victim protection. The "spared stash keys climb through probing" interaction from the hit-rate refutation case did not happen. That fits the rationale: the window acts on the get-path probing arm and the protection acts at stash exit, so the two levers stack without clashing.

## Hurt

- per-op-time: get p50 and get p99 match the parent exactly, so the narrower window bought no get latency. The hypothesis's main claim did not land. This points to the probing arm's hottest-hit promote/evict branch firing too rarely to move get percentiles. Put p99 also rose clearly, even though the change never touches the put path. The likely cause is environmental noise. One possible code-side cause: fewer probing promotions could leave more buckets full, which sends more puts down the stash-full path with its extra two-slot check. Neither cause can be confirmed from this data.

## Takeaway

`PROBE_WINDOW` stacks safely on top of bounded stash protection: hit-rate is unchanged and the hm_0 gain is intact. But it is not a get latency knob. The get percentiles did not move at all. This matches the hypothesis's latency-side refutation case: the window should be left alone and dropped as a latency lever. A final verdict needs the same-session H01 control numbers, which this comparison does not include. The put p99 jump should be rechecked on a quiet host before anyone blames the window for it. Caveat: I could not list or open the overlay source in this session. This reading assumes H02 was implemented as written: only the window const changed, with no layout change, which the unchanged 24-byte metadata supports.
