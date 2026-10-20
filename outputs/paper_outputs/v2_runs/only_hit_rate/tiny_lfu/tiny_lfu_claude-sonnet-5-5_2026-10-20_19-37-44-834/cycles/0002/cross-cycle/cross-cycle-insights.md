## Recurring wins

- None robust yet. The only gain so far was a marginal hit-rate improvement from a window plus segmented main region (cycle 1), and a later larger-window variant did not extend it.
- Compacting the sketch (narrower counters, cheaper hashing) cost no hit-rate in cycle 1. It is a candidate cost win, but cost is still unmeasured.

## Recurring failures

- Mechanisms that discard or hide frequency information (aging/decay, larger window, doorkeeper) lost hit-rate on workload-a in cycle 2. The workload looks stable and frequency-driven.
- Hit-rate-only tuning on a single trace: changes move the metric only slightly in either direction, so Winner labels may only reflect ranking among near-ties.
- Fitness vectors omit metadata-size and per-op-time in both cycles, so cost-oriented ideas (e.g. compact sketch) cannot be judged. This is the most persistent gap.
- Infrastructure fragility: a simulator build failed on a network rate limit (dependency download) and rejected an idea that had nothing wrong with it.

## Open questions

- Do the window/segmentation and compact-sketch variants change metadata size or per-op time, even though hit-rate barely moves?
- Does aging pay off on a trace with real popularity shifts or a longer sample period? Pure aging has only been tested on top of the windowed design.
- Does a doorkeeper help if admission is adjusted for its offset, or on a workload with sketch pollution?
- Are the small hit-rate differences real or noise? Other traces and cache sizes are needed.
- Can the compact sketch be combined with the window or aging while preserving frequency information?

## Per-policy notes

- TinyLFU, baseline sketch-gated admission: it already captures most of the available benefit on workload-a. Further structural tweaks give negligible or negative returns.
- Windowed/segmented variant: marginally positive at the smaller window. Enlarging the window hurt, probably because it dilutes main-region protection.
- Compact u8 sketch with single-hash and conservative update: neutral on hit-rate, so the baseline sketch looks over-provisioned.
- Aging (periodic halving): rejected once for infrastructure reasons, then neutral-to-negative on workload-a. Its value needs a drifting trace.
- Doorkeeper: slightly negative on workload-a, probably because first sightings are hidden from the counters.
